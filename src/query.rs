use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use crate::{Filter, FilterError, Filters, Query, QueryError, ValueError};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use surrealdb::{
    engine::any::Any,
    sql::{
        statements::SelectStatement, value, Cond, Expression, Idiom, Limit, Number, Operator,
        Order, Orders, Start, Table, Value, Values,
    },
};

#[derive(Debug, Clone)]
pub struct SOrder(pub Order);

impl From<SOrder> for Order {
    fn from(s: SOrder) -> Self {
        s.0
    }
}

impl<S> From<(S, bool)> for SOrder
where
    S: Into<String>,
{
    fn from((v, d): (S, bool)) -> Self {
        Self(Order {
            order: Idiom::from(v.into()),
            direction: d,
            random: false,
            collate: false,
            numeric: false,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SOrders(pub Vec<SOrder>);

impl SOrders {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl From<SOrders> for Orders {
    fn from(s: SOrders) -> Self {
        Self(s.0.into_iter().map(Into::<Order>::into).collect())
    }
}

#[derive(Debug, Clone)]
pub struct SCond(pub Cond);

impl Default for SCond {
    fn default() -> Self {
        Self(Cond(Value::Expression(Box::new(Expression::default()))))
    }
}

impl From<SCond> for Cond {
    fn from(s: SCond) -> Self {
        s.0
    }
}

impl From<SCond> for Value {
    fn from(c: SCond) -> Self {
        match c.0 {
            Cond(v) => v,
        }
    }
}

impl Default for SOrder {
    fn default() -> Self {
        Self(Order {
            order: Idiom::from("dateCreated".to_string()),
            direction: true,
            random: false,
            collate: false,
            numeric: false,
        })
    }
}

impl From<Expression> for SCond {
    fn from(e: Expression) -> Self {
        Self(Cond(Value::Expression(Box::new(e))))
    }
}

impl SCond {
    pub fn and(self, c: impl Into<Cond>) -> Self {
        Self(Cond(Value::Expression(Box::new(Expression::Binary {
            l: self.into(),
            o: Operator::And,
            r: SCond(c.into()).into(),
        }))))
    }

    pub fn or(self, c: impl Into<Cond>) -> Self {
        Self(Cond(Value::Expression(Box::new(Expression::Binary {
            l: self.into(),
            o: Operator::Or,
            r: SCond(c.into()).into(),
        }))))
    }

    pub fn to_value(self) -> Value {
        match self.0.into() {
            Cond(v) => v,
        }
    }
}

pub struct SurrealQuery<U>
where
    U: DeserializeOwned,
{
    binds: BTreeMap<String, Filter>,

    db: Arc<surrealdb::Surreal<Any>>,

    stmt: SelectStatement,
    u_type: PhantomData<U>,
}

impl<U> SurrealQuery<U>
where
    U: DeserializeOwned,
{
    pub fn new(db: Arc<surrealdb::Surreal<Any>>) -> Self {
        Self {
            db: db.into(),
            binds: BTreeMap::<String, Filter>::new(),
            stmt: SelectStatement::default(),
            u_type: PhantomData,
        }
    }
}

#[async_trait]
impl<U> Query<U> for SurrealQuery<U>
where
    U: DeserializeOwned + Sync + Send,
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
        self.stmt.what = Values(vec![Table::from(table.into()).into()]);

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
        let cond = match filters
            .clone()
            .into_iter()
            .map(|f| -> Result<SCond, ValueError> {
                match f
                    .into_iter()
                    .map(|(k, v)| match v {
                        Filter::Equal(_) => Ok(Expression::Binary {
                            l: to_value(k.clone())?,
                            o: Operator::Equal,
                            r: to_value(format!("${}", k))?,
                        }),
                        Filter::Range(f) => Ok(Expression::Binary {
                            l: to_value(k.clone())?,
                            o: match (f.lt, f.gt, f.lte, f.gte) {
                                (Some(_), _, _, _) => Operator::LessThan,
                                (_, Some(_), _, _) => Operator::MoreThan,
                                (_, _, Some(_), _) => Operator::LessThanOrEqual,
                                (_, _, _, Some(_)) => Operator::MoreThanOrEqual,
                                _ => Operator::Equal,
                            },
                            r: to_value(format!("${}", k))?,
                        }),
                        Filter::OneOf(_) => Ok(Expression::Binary {
                            l: to_value(k.clone())?,
                            o: Operator::Inside,
                            r: to_value(format!("${}", k))?,
                        }),
                    })
                    .filter_map(|e: Result<Expression, ValueError>| e.ok())
                    .map(Into::<SCond>::into)
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
            .reduce(|acc, e| acc.or(e))
        {
            Some(cond) => cond,
            None => SCond::default(),
        };

        self.stmt.cond = Some(cond.into());
        self.binds = filters
            .clone()
            .into_iter()
            .flatten()
            .map(|(k, f)| (k, f))
            .collect();

        Ok(self)
    }

    // page sets the pagination for this query using the limit and message_cid fields.
    // The limit field is the number of messages to return in the query, and
    // the message_cid field is the cid of the message to start the query from.
    // If the message_cid field is not set, the query will start from the beginning.
    // If the limit field is not set, the query will return all messages.
    fn page(&mut self, pagination: Option<crate::Pagination>) -> &mut Self {
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                self.stmt.limit = Some(Limit(Value::Number(Number::from(l))));
            }

            if let Some(m) = p.message_cid {
                self.stmt.start = Some(Start(Value::Idiom(Idiom::from(m))));
            }
        }

        self
    }

    // sort sets the sort order for this query. The argument to this function is a
    // MessageSort struct, which contains the fields to sort on and the direction to sort.
    // If the MessageSort struct is not set, the query will return messages in the order
    // they were published.
    fn sort(&mut self, sort: Option<crate::MessageSort>) -> &mut Self {
        if let Some(s) = sort {
            let mut orders = SOrders::new();

            if let Some(d) = s.date_created {
                orders.0.push(("dateCreated", d.to_bool()).into());
            }

            if let Some(d) = s.date_published {
                orders.0.push(("datePublished", d.to_bool()).into());
            }

            if let Some(d) = s.timestamp {
                orders.0.push(("messageTimestamp", d.to_bool()).into());
            }

            self.stmt.order = Some(orders.into());
        }

        self
    }

    async fn query(&self) -> Result<Vec<U>, QueryError> {
        let mut stmt = self.stmt.clone();
        stmt.expr.0.push(surrealdb::sql::Field::All);

        let mut q = self.db.query(stmt.clone()).bind(self.binds.clone()).await?;

        let res: Vec<U> = q.take(0)?;
        web_sys::console::log_1(&format!("Query: {:?}", stmt.clone().to_string()).into());

        Ok(res)
    }
}

pub(crate) fn to_value(v: impl Into<String>) -> Result<Value, ValueError> {
    Ok(value(Into::<String>::into(v).as_str())?)
}
