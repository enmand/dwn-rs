#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum IndexValue {
    Bool(bool),
    String(String),
}

impl From<bool> for IndexValue {
    fn from(b: bool) -> Self {
        IndexValue::Bool(b.into())
    }
}

impl From<&str> for IndexValue {
    fn from(s: &str) -> Self {
        IndexValue::String(s.into())
    }
}
