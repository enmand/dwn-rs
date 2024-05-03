use std::marker::PhantomData;
use std::{
    fmt::Debug,
    ops::{Bound, RangeBounds},
};

use async_trait::async_trait;
use dwn_rs_core::MapValue;
use serde::de::DeserializeOwned;
use surrealdb::sql::{value as surreal_value, Cond, Function, Idiom, Subquery};
use surrealdb::{
    engine::any::Any,
    sql::{statements::SelectStatement, Expression, Limit, Number, Operator, Table, Value, Values},
};

use super::expr::{SCond, SOrders};
use crate::{
    filters::{
        errors::{FilterError, QueryError, ValueError},
        filters::{Filter, Filters},
        query::{Cursor, CursorValue, Pagination, Query, SortDirection},
        Directional,
    },
    FilterKey, Set,
};
use crate::{Alias, Ordorable};

pub struct SurrealQuery<U, T>
where
    U: DeserializeOwned,
    T: Directional + Default + Ordorable + Sync + Copy,
{
    binds: MapValue,

    db: surrealdb::Surreal<Any>,

    stmt: SelectStatement,
    from: String,
    limit: Option<u32>,
    order: Option<T>,
    sort_direction: Option<SortDirection>,
    cursor: Option<Cursor>,
    always_cursor: bool,
    u_type: PhantomData<U>,
}

impl<U, T> SurrealQuery<U, T>
where
    U: DeserializeOwned,
    T: Directional + Default + Ordorable + Sync + Copy,
{
    pub fn new(db: surrealdb::Surreal<Any>) -> Self {
        Self {
            db,
            binds: MapValue::new(),
            stmt: SelectStatement::default(),
            from: String::default(),
            limit: None,
            order: Some(T::default()),
            sort_direction: Some(*T::default().get_direction()),
            cursor: None,
            always_cursor: false,
            u_type: PhantomData,
        }
    }
}

#[async_trait]
impl<U, T> Query<U, T> for SurrealQuery<U, T>
where
    U: CursorValue<T> + DeserializeOwned + Sync + Send + Debug,
    T: Directional + Default + Ordorable + Sync + Copy,
{
    /// from sets the table to query from.
    ///
    /// The argument to this function is the table name to query from.
    /// This function will overwrite any previous table set on this query.
    /// The table name must be a valid table name in the database.
    /// If the table name is not a valid table name in the database, an error will be returned.
    fn from<S>(&mut self, table: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.from = table.into();

        let what = Function::Normal(
            "type::table".into(),
            vec![format!("{}", Table::from(self.from.clone())).into()],
        )
        .into();

        self.stmt.what = Values(vec![what]);

        self
    }

    /// filter sets the query filters for this query.
    ///
    /// The arguments to this function are the set of Filters to apply. Filters that
    /// are passed in will be applied as an OR operation. Filters that are passed in
    /// as a Vec will be applied as an AND operation.
    ///
    /// This function will overwrite any previous filters set on this query.
    fn filter(&mut self, filters: &Filters) -> Result<&mut Self, FilterError> {
        let filters = Into::<Set<Alias>>::into(filters.clone() as Filters)
            .into_iter()
            .filter_map(|f| {
                f.into_iter()
                    .map(|((fk, alias), val)| {
                        let k = match fk {
                            FilterKey::Tag(_) => format!("tags.{}", fk),
                            _ => fk.to_string(),
                        };

                        match val {
                            Filter::Prefix(v) => {
                                self.binds.insert(alias.clone(), v);

                                Ok(Value::Function(Box::new(Function::Normal(
                                    "string::startsWith".into(),
                                    vec![Idiom::from(k).into(), format!("${}", alias).into()],
                                )))
                                .into())
                            }
                            Filter::Equal(v) => {
                                self.binds.insert(alias.clone(), v);

                                Ok((k, Operator::Equal, format!("${}", alias)).try_into()?)
                            }
                            Filter::Range(lower, upper) => {
                                let (cond, binds) =
                                    handle_range_filter((k, alias), (lower, upper))?;

                                self.binds.extend(binds);
                                Ok(cond)
                            }
                            Filter::OneOf(v) => {
                                self.binds.insert(alias.clone(), v.into());

                                Ok((k, Operator::Inside, format!("${}", alias)).try_into()?)
                            }
                        }
                    })
                    .reduce(
                        |acc: Result<SCond, FilterError>, e: Result<SCond, FilterError>| {
                            Ok(acc?.and(e?))
                        },
                    )
            })
            .map(|c| {
                Ok(SCond(Cond(Value::Subquery(Box::new(Subquery::Value(
                    c?.into(),
                ))))))
            })
            .reduce(
                |acc: Result<SCond, FilterError>, e: Result<SCond, FilterError>| Ok(acc?.or(e?)),
            );

        if let Some(filters) = filters {
            self.stmt.cond = Some(filters?.into());
        }

        Ok(self)
    }

    // page sets the pagination for this query using the limit and message_cid fields.
    // The limit field is the number of messages to return in the query, and
    // the message_cid field is the cid of the message to start the query from.
    // If the message_cid field is not set, the query will start from the beginning.
    // If the limit field is not set, the query will return all messages.
    fn page(&mut self, pagination: Option<Pagination>) -> &mut Self {
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                self.limit = Some(l);
                self.stmt.limit = Some(Limit(Value::Number(Number::from(l + 1))));
            }

            if let Some(c) = p.cursor {
                self.cursor = Some(c);
            }
        }

        self
    }

    // always_cursor forces the query builder to return a cursor for the last element, even if the
    // limit is not set, or there are no further messages
    fn always_cursor(&mut self) -> &mut Self {
        self.always_cursor = true;
        self
    }

    // sort sets the sort order for this query. The argument to this function is a
    // MessageSort struct, which contains the fields to sort on and the direction to sort.
    // If the MessageSort struct is not set, the query will return messages in the order
    // they were published.
    fn sort(&mut self, sort: Option<T>) -> &mut Self {
        self.order = match sort {
            Some(s) => Some(s),
            None => self.order,
        };

        if let Some(o) = self.order {
            let direction = o.get_direction();
            let order: SOrders = o.to_order().into();

            self.stmt.order = Some(order.into());
            self.sort_direction = Some(*direction);
        }

        self
    }

    async fn query(&self) -> Result<(Vec<U>, Option<Cursor>), QueryError> {
        let mut stmt = self.stmt.clone();
        stmt.expr.0.push(surrealdb::sql::Field::All);

        let mut binds = self.binds.clone();

        stmt.cond = match cursor_cond(&self.stmt, self.cursor.clone(), self.order)? {
            Some((conds, mut cursor_binds)) => {
                binds.append(&mut cursor_binds);

                if let Some(filters) = stmt.cond {
                    Some(Cond(
                        Expression::Binary {
                            l: filters.0,
                            o: Operator::And,
                            r: conds.0,
                        }
                        .into(),
                    ))
                } else {
                    Some(conds)
                }
            }
            None => stmt.cond,
        };

        let mut q = self
            .db
            .query(stmt.clone())
            .bind(&binds)
            .await
            .map_err(|e| QueryError::DbError(e.to_string()))?;

        let mut res: Vec<U> = q.take(0).map_err(|e| QueryError::DbError(e.to_string()))?;

        let mut limit = self.limit;
        if self.always_cursor {
            limit = Some(0);
        }

        let last_cursor_value = if let (Some(l), Some(o)) = (limit, self.order) {
            if res.len() as u32 > l {
                if !self.always_cursor {
                    res.pop();
                }

                res.last().map(|r| Cursor {
                    cursor: r.cid(),
                    value: Some(r.cursor_value(o).clone()),
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok((res, last_cursor_value))
    }
}

pub(super) fn value(s: &str) -> Result<Value, ValueError> {
    surreal_value(s).map_err(|e| ValueError::InvalidValue(e.to_string()))
}

fn handle_range_filter<R>(
    (fk, alias): (String, String),
    range: R,
) -> Result<(SCond, MapValue), FilterError>
where
    R: RangeBounds<dwn_rs_core::Value> + Debug,
{
    let lower = match range.start_bound() {
        Bound::Included(l) => Some((Operator::MoreThanOrEqual, l)),
        Bound::Excluded(l) => Some((Operator::MoreThan, l)),
        _ => None,
    };

    let upper = match range.end_bound() {
        Bound::Included(u) => Some((Operator::LessThanOrEqual, u)),
        Bound::Excluded(u) => Some((Operator::LessThan, u)),
        _ => None,
    };

    match (lower, upper) {
        (Some((l_op, l)), Some((u_op, u))) => {
            let l_cond = SCond::try_from((fk.clone(), l_op, format!("${}_lower", alias)))?;
            let u_cond = SCond::try_from((fk.clone(), u_op, format!("${}_upper", alias)))?;

            let mut binds = MapValue::new();
            binds.insert(format!("{}_lower", alias), l.clone());
            binds.insert(format!("{}_upper", alias), u.clone());

            Ok((l_cond.and(u_cond), binds))
        }
        (Some((l_op, l)), None) => {
            let l_cond = SCond::try_from((fk, l_op, format!("${}_lower", alias)))?;

            let mut binds = MapValue::new();
            binds.insert(format!("{}_lower", alias), l.clone());

            Ok((l_cond, binds))
        }
        (None, Some((u_op, u))) => {
            let u_cond = SCond::try_from((fk, u_op, format!("${}_upper", alias)))?;

            let mut binds = MapValue::new();
            binds.insert(format!("{}_upper", alias), u.clone());

            Ok((u_cond, binds))
        }
        _ => Err(FilterError::UnparseableFilter("Invalid range".to_owned())),
    }
}

/// Create the cursor condition for the query to SurrealDB using the provided cursor and
/// ordering. If the cursor is set, the condition will be set to the value of the cursor
/// and the ordering will be set to the direction of the ordering. If the cursor is not
/// set, the condition will be set to None.
fn cursor_cond<T: Ordorable + Directional>(
    stmt: &SelectStatement,
    cursor: Option<Cursor>,
    order: Option<T>,
) -> Result<Option<(Cond, MapValue)>, ValueError> {
    if let (
        Some(Cursor {
            cursor: c,
            value: Some(v),
        }),
        Some(o),
    ) = (&cursor, &order)
    {
        let mut binds = MapValue::new();

        // get the direction of the sort, and set the operator to MoreThan if ASC, LessThan if DESC
        let op = match o.get_direction() {
            SortDirection::Ascending => Operator::MoreThan,
            SortDirection::Descending => Operator::LessThan,
        };

        binds.insert("_cursor_val".to_owned(), v.clone());
        binds.insert(
            "_cursor".to_owned(),
            dwn_rs_core::value::Value::String(c.to_string()),
        );

        let cur_cond = Value::Subquery(Box::new(Subquery::Value(
            Expression::Binary {
                l: Value::Subquery(Box::new(Subquery::Value(
                    Expression::Binary {
                        l: Expression::Binary {
                            l: stmt.order.clone().unwrap().0[0].order.clone().into(),
                            o: Operator::Equal,
                            r: value("$_cursor_val")?,
                        }
                        .into(),
                        o: Operator::And,
                        r: Expression::Binary {
                            l: Idiom::from("cid".to_string()).into(),
                            o: op.clone(),
                            r: value("$_cursor")?,
                        }
                        .into(),
                    }
                    .into(),
                ))),
                o: Operator::Or,
                r: Expression::Binary {
                    l: stmt.order.clone().unwrap().0[0].order.clone().into(),
                    o: op,
                    r: value("$_cursor_val")?,
                }
                .into(),
            }
            .into(),
        )));

        return Ok(Some((Cond(cur_cond), binds)));
    }

    Ok(None)
}
