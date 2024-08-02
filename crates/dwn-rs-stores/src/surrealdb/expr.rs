use surrealdb::sql::{Cond, Expression, Idiom, Operator, Order, Orders, Value};

use super::query::value;
use dwn_rs_core::filters::errors::ValueError;

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
        let mut o = Order::default();
        o.order = Idiom::from(v.into());
        o.direction = d;

        Self(o)
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

impl From<(&str, bool)> for SOrders {
    fn from((v, d): (&str, bool)) -> Self {
        Self(vec![SOrder::from((v, d))])
    }
}

impl From<Vec<(&str, bool)>> for SOrders {
    fn from(v: Vec<(&str, bool)>) -> Self {
        Self(v.iter().map(|(v, d)| SOrder::from((*v, *d))).collect())
    }
}

impl From<SOrders> for Orders {
    fn from(s: SOrders) -> Self {
        let mut o = Orders::default();
        o.0 = s.0.into_iter().map(Into::<Order>::into).collect();

        o
    }
}

/// SCond is a wrapper around Cond which allows for more ergonomic construction of Cond
/// structs.
#[derive(Debug, Clone, Default)]
pub struct SCond(pub Cond);

impl SCond {
    pub fn new() -> Self {
        let mut cond = Cond::default();
        cond.0 = Value::None;
        Self(cond)
    }
}

impl From<SCond> for Cond {
    fn from(s: SCond) -> Self {
        s.0
    }
}

impl From<SCond> for Value {
    fn from(c: SCond) -> Self {
        c.0 .0
    }
}

impl TryFrom<(String, Operator, String)> for SCond {
    type Error = ValueError;
    fn try_from((l, o, r): (String, Operator, String)) -> Result<Self, Self::Error> {
        let mut cond = Cond::default();

        cond.0 = Value::Expression(Box::new(Expression::Binary {
            l: value(l.as_str())?,
            o,
            r: value(r.as_str())?,
        }));

        Ok(Self(cond))
    }
}

impl TryFrom<(String, Operator, Value)> for SCond {
    type Error = ValueError;
    fn try_from((l, o, r): (String, Operator, Value)) -> Result<Self, Self::Error> {
        let mut cond = Cond::default();

        cond.0 = Value::Expression(Box::new(Expression::Binary {
            l: value(l.as_str())?,
            o,
            r,
        }));

        Ok(Self(cond))
    }
}

impl From<Value> for SCond {
    fn from(v: Value) -> Self {
        let mut cond = Cond::default();
        cond.0 = v;

        Self(cond)
    }
}

impl From<(SCond, Operator, SCond)> for SCond {
    fn from((l, o, r): (SCond, Operator, SCond)) -> Self {
        let mut cond = Cond::default();
        cond.0 = Value::Expression(Box::new(Expression::Binary {
            l: l.into(),
            o,
            r: r.into(),
        }));

        Self(cond)
    }
}

impl From<Expression> for SCond {
    fn from(e: Expression) -> Self {
        let mut cond = Cond::default();
        cond.0 = Value::Expression(Box::new(e));

        Self(cond)
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
        &self.0 .0
    }
}
