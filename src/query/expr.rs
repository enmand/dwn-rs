use crate::ValueError;
use surrealdb::sql::{value, Cond, Expression, Idiom, Operator, Order, Orders, Value};

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

/// SCond is a wrapper around Cond which allows for more ergonomic construction of Cond
/// structs.
#[derive(Debug, Clone)]
pub struct SCond(pub Cond);

impl SCond {
    pub fn new() -> Self {
        Self(Cond(Value::None))
    }
}

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

/// to value converts a string into a Value.
pub(crate) fn to_value(v: impl Into<String>) -> Result<Value, ValueError> {
    Ok(value(Into::<String>::into(v).as_str())?)
}
