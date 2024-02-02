use surrealdb::sql::{Cond, Expression, Idiom, Operator, Order, Orders, Value};

use super::query::value;
use crate::{filters::errors::ValueError, MessageSort};

/// SOrder is a wrapper around Order which allows for more ergonomic construction of Order
/// structs.
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

impl<S> From<(S, bool, bool)> for SOrder
where
    S: Into<String>,
{
    fn from((v, d, c): (S, bool, bool)) -> Self {
        Self(Order {
            order: Idiom::from(v.into()),
            direction: d,
            random: false,
            collate: c,
            numeric: false,
        })
    }
}

/// SOrders is a wrapper around Orders which allows for more ergonomic construction of Orders
/// vecs.
#[derive(Debug, Clone)]
pub struct SOrders(pub Vec<SOrder>);

impl Default for SOrders {
    fn default() -> Self {
        Self::new()
    }
}

impl SOrders {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, o: impl Into<SOrder>) -> Self {
        self.0.push(o.into());

        self.to_owned()
    }

    pub fn set(&mut self, o: impl Into<SOrder>) {
        self.0 = vec![o.into()];
    }
}

impl From<SOrders> for Orders {
    fn from(s: SOrders) -> Self {
        Self(s.0.into_iter().map(Into::<Order>::into).collect())
    }
}

/// Ordable is a trait that allows for the conversion of a type into an Order.
pub trait Ordable {
    fn to_order(self) -> SOrders;
}

impl Ordable for MessageSort {
    fn to_order(self) -> SOrders {
        match self {
            MessageSort::DateCreated(direction) => {
                SOrders::new().push(("dateCreated", direction.to_bool()))
            }
            MessageSort::DatePublished(direction) => {
                SOrders::new().push(("datePublished", direction.to_bool()))
            }
            MessageSort::Timestamp(direction) => {
                SOrders::new().push(("messageTimestamp", direction.to_bool()))
            }
        }
    }
}

/// SCond is a wrapper around Cond which allows for more ergonomic construction of Cond
/// structs.
#[derive(Debug, Clone, Default)]
pub struct SCond(pub Cond);

impl SCond {
    pub fn new() -> Self {
        Self(Cond(Value::None))
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

impl TryFrom<(String, Operator, String)> for SCond {
    type Error = ValueError;
    fn try_from((l, o, r): (String, Operator, String)) -> Result<Self, Self::Error> {
        Ok(Self(Cond(Value::Expression(Box::new(
            Expression::Binary {
                l: value(l.as_str())?,
                o,
                r: value(r.as_str())?,
            },
        )))))
    }
}

impl From<(SCond, Operator, SCond)> for SCond {
    fn from((l, o, r): (SCond, Operator, SCond)) -> Self {
        Self(Cond(Value::Expression(Box::new(Expression::Binary {
            l: l.into(),
            o,
            r: r.into(),
        }))))
    }
}

impl From<Expression> for SCond {
    fn from(e: Expression) -> Self {
        Self(Cond(Value::Expression(Box::new(e))))
    }
}

impl SCond {
    pub fn and<C>(self, c: C) -> Self
    where
        C: Into<Cond>,
    {
        (self, Operator::And, SCond(c.into())).into()
    }

    pub fn or<C>(self, c: C) -> Self
    where
        C: Into<Cond>,
    {
        (self, Operator::Or, SCond(c.into())).into()
    }

    pub fn to_value(&self) -> &Value {
        match &self.0 {
            Cond(v) => v,
        }
    }
}
