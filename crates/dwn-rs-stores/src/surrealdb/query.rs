use std::fmt::Debug;
use std::ops::Bound;
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use surrealdb::sql::{value as surreal_value, Cond, Function, Idiom, Subquery};
use surrealdb::{
    engine::any::Any,
    sql::{statements::SelectStatement, Expression, Limit, Number, Operator, Table, Value, Values},
};

use super::expr::{Ordable, SCond};
use crate::filters::{
    errors::{FilterError, QueryError, ValueError},
    filters::{Filter, Filters},
    query::{Cursor, CursorValue, MessageSort, Pagination, Query, SortDirection},
};

pub struct SurrealQuery<U>
where
    U: DeserializeOwned,
{
    binds: BTreeMap<String, crate::filters::value::Value>,

    db: Arc<surrealdb::Surreal<Any>>,

    stmt: SelectStatement,
    from: String,
    limit: Option<u32>,
    order: Option<MessageSort>,
    sort_direction: Option<SortDirection>,
    cursor: Option<Cursor>,
    u_type: PhantomData<U>,
}

impl<U> SurrealQuery<U>
where
    U: DeserializeOwned,
{
    pub fn new(db: Arc<surrealdb::Surreal<Any>>) -> Self {
        Self {
            db,
            binds: BTreeMap::<String, crate::filters::value::Value>::new(),
            stmt: SelectStatement::default(),
            from: String::default(),
            limit: None,
            order: Some(MessageSort::default()),
            sort_direction: Some(MessageSort::default().get_direction()),
            cursor: None,
            u_type: PhantomData,
        }
    }
}

#[async_trait]
impl<U> Query<U> for SurrealQuery<U>
where
    U: CursorValue + DeserializeOwned + Sync + Send + Debug,
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
        self.stmt.cond = filters
            .clone()
            .into_iter()
            .enumerate()
            .map(|(s, k)| -> Vec<((String, String), Filter)> {
                k.into_iter()
                    .map(|(k, v)| ((k.clone(), format!("{}_{}", k, &s)), v))
                    .collect()
            })
            .map(|f| -> Result<SCond, ValueError> {
                match f
                    .into_iter()
                    .map(|((k, var), val)| match val {
                        Filter::Equal(v) => {
                            self.binds.insert(var.clone(), v);

                            Ok(SCond::try_from((k, Operator::Equal, format!("${}", var)))?)
                        }
                        Filter::Range(lower, upper) => Ok(
                            match (
                                match lower {
                                    Bound::Included(l) => Some((
                                        Operator::MoreThanOrEqual,
                                        format!("{}_lower", var),
                                        l,
                                    )),
                                    Bound::Excluded(l) => {
                                        Some((Operator::MoreThan, format!("{}_lower", var), l))
                                    }
                                    _ => None,
                                },
                                match upper {
                                    Bound::Included(u) => Some((
                                        Operator::LessThanOrEqual,
                                        format!("{}_upper", var),
                                        u,
                                    )),
                                    Bound::Excluded(u) => {
                                        Some((Operator::LessThan, format!("{}_upper", var), u))
                                    }
                                    _ => None,
                                },
                            ) {
                                (Some(l), Some(u)) => {
                                    self.binds.insert(l.1.clone(), l.2);
                                    self.binds.insert(u.1.clone(), u.2);

                                    SCond::try_from((k.clone(), l.0, format!("${}", l.1)))?
                                        .and(SCond::try_from((k, u.0, format!("${}", u.1)))?)
                                }
                                (Some(l), None) => {
                                    self.binds.insert(l.1.to_string(), l.2);

                                    SCond::try_from((k, l.0, format!("${}", l.1)))?
                                }
                                (None, Some(u)) => {
                                    self.binds.insert(u.1.to_string(), u.2);

                                    SCond::try_from((k, u.0, format!("${}", u.1)))?
                                }
                                (None, None) => {
                                    return Err(FilterError::UnparseableFilter(
                                        "Could not parse filter".to_owned(),
                                    )
                                    .into())
                                }
                            },
                        ),
                        Filter::OneOf(v) => {
                            self.binds
                                .insert(var.clone(), crate::filters::value::Value::Array(v));

                            Ok(SCond::try_from((k, Operator::Inside, format!("${}", var)))?)
                        }
                    })
                    .filter_map(|e: Result<SCond, ValueError>| e.ok())
                    .reduce(|acc: SCond, e: SCond| acc.and(e))
                {
                    Some(cond) => Ok(cond),
                    None => Err(FilterError::UnparseableFilter(
                        "Could not parse filter".to_owned(),
                    )
                    .into()),
                }
            })
            .filter_map(|e| e.ok())
            .map(|c| SCond(Cond(Value::Subquery(Box::new(Subquery::Value(c.into()))))))
            .reduce(|acc, e| acc.or(e))
            .map(|c| c.into());

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

    // sort sets the sort order for this query. The argument to this function is a
    // MessageSort struct, which contains the fields to sort on and the direction to sort.
    // If the MessageSort struct is not set, the query will return messages in the order
    // they were published.
    fn sort(&mut self, sort: Option<MessageSort>) -> &mut Self {
        self.order = match sort {
            Some(s) => Some(s),
            None => self.order,
        };

        if let Some(o) = self.order {
            let direction = o.get_direction();
            let mut order = o.to_order();
            order.push(("cid", direction.to_bool(), false));

            self.stmt.order = Some(order.into());
            self.sort_direction = Some(direction);
        }

        self
    }

    async fn query(&self) -> Result<(Vec<U>, Option<Cursor>), QueryError> {
        let mut stmt = self.stmt.clone();
        stmt.expr.0.push(surrealdb::sql::Field::All);

        let mut binds = self.binds.clone();

        if let (
            Some(Cursor {
                cursor: c,
                value: Some(v),
            }),
            Some(o),
        ) = (&self.cursor, &self.order)
        {
            // get the direction of the sort, and set the operator to MoreThan if ASC, LessThan if DESC
            let op = match o.get_direction() {
                SortDirection::Ascending => Operator::MoreThan,
                SortDirection::Descending => Operator::LessThan,
            };

            binds.insert("_cursor_val".to_owned(), v.clone());
            binds.insert(
                "_cursor".to_owned(),
                crate::filters::value::Value::String(c.to_string()),
            );

            let cur_cond = Value::Subquery(Box::new(Subquery::Value(
                Expression::Binary {
                    l: Value::Subquery(Box::new(Subquery::Value(
                        Expression::Binary {
                            l: Expression::Binary {
                                l: self.stmt.order.clone().unwrap().0[0].order.clone().into(),
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
                        l: self.stmt.order.clone().unwrap().0[0].order.clone().into(),
                        o: op,
                        r: value("$_cursor_val")?,
                    }
                    .into(),
                }
                .into(),
            )));

            if let Some(cond) = stmt.cond {
                stmt.cond = Some(Cond(
                    Expression::Binary {
                        l: cond.0,
                        o: Operator::And,
                        r: cur_cond,
                    }
                    .into(),
                ));
            } else {
                stmt.cond = Some(Cond(cur_cond));
            }
        }

        let mut q = self
            .db
            .query(stmt.clone())
            .bind(&binds)
            .await
            .map_err(|e| QueryError::DbError(e.to_string()))?;
        let mut res: Vec<U> = q.take(0).map_err(|e| QueryError::DbError(e.to_string()))?;

        let last_cursor_value = if let (Some(l), Some(o)) = (self.limit, self.order) {
            if res.len() as u32 > l {
                res.pop();

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
