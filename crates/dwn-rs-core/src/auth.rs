use crate::MapValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct JWS {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<SignatureEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<MapValue>,
    #[serde(flatten)]
    pub extra: MapValue,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct SignatureEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(flatten)]
    pub extra: MapValue,
}
