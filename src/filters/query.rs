use std::collections::HashMap;

use crate::{Filter, Filters};

// Trait for implementing Filters
pub trait Query {
    fn query(&self) -> (String, HashMap<String, Filter>);
}

impl Query for Filters {
    fn query(&self) -> (String, HashMap<String, Filter>) {
        let mut queries = Vec::<String>::new();
        let mut binds = HashMap::<String, Filter>::new();

        for expr in self.filters.iter() {
            let mut query = Vec::<String>::new();
            for (k, v) in expr.iter() {
                match v {
                    Filter::Equal(f) => {
                        query.push(format!("{} = {}", k, f));
                    }
                    Filter::Range(f) => query.push(f.range_with(k)),
                    Filter::OneOf(f) => {
                        let f = f
                            .iter()
                            .map(|v| format!("{}", v))
                            .collect::<Vec<String>>()
                            .join(", ");
                        query.push(format!("{} ∈ [{}]", k, f));
                    }
                }
                binds.insert(k.clone(), v.clone());
            }
            queries.push(query.join(" AND "))
        }

        (queries.join(" OR "), binds)
    }
}
    }
}
