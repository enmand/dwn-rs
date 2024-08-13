use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_dids_core::DIDBuf;
use ssi_jwk::JWK;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ConfigureDescriptor {
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub definition: ProtocolDefinition,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProtocolDefinition {
    pub protocol: String,
    pub published: bool,
    pub types: BTreeMap<String, Option<ProtocolType>>,
    pub structure: BTreeMap<String, ProtocolRule>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProtocolType {
    pub schema: Option<String>,
    #[serde(rename = "dataFormats")]
    pub data_formats: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProtocolRule {
    #[serde(rename = "$encryption")]
    pub encryption: Option<Encryption>,
    #[serde(rename = "$actions", default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
    #[serde(rename = "$role")]
    pub role: Option<bool>,
    #[serde(rename = "$size")]
    pub size: Option<Size>,
    #[serde(rename = "$tags")]
    pub tags: Option<Tags>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, ProtocolRule>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Encryption {
    #[serde(rename = "rootKeyId")]
    pub root_key_id: String,
    #[serde(rename = "publicKeyJwk")]
    pub public_key_jwk: JWK,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Who {
    #[serde(rename = "anyone")]
    Anyone,
    #[serde(rename = "author")]
    Author,
    #[serde(rename = "recipient")]
    Recipient,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Can {
    #[serde(rename = "co-delete")]
    CoDelete,
    #[serde(rename = "co-prune")]
    CoPrune,
    #[serde(rename = "co-update")]
    CoUpdate,
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "prune")]
    Prune,
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "subscribe")]
    Subscribe,
    #[serde(rename = "query")]
    Query,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Action {
    Who {
        who: Who,
        of: Option<String>,
        can: Vec<Can>,
    },
    Role {
        role: String,
        can: Vec<Can>,
    },
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Size {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Tags {
    #[serde(
        rename = "$requiredTags",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub required_tags: Vec<String>,
    #[serde(rename = "$allowUndefinedTags")]
    pub allow_undefined_tags: Option<bool>,
    #[serde(flatten)]
    pub tags: BTreeMap<String, ProvidedTags>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TagType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "array")]
    Array,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ItemType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "integer")]
    Integer,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProvidedTags {
    #[serde(rename = "type")]
    pub tag_type: TagType,
    pub items: Option<TagItems>,
    pub contains: Option<TagContains>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<usize>,
    #[serde(rename = "minLength")]
    pub min_length: Option<usize>,
    pub minimum: Option<usize>,
    pub maximum: Option<usize>,
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<usize>,
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<usize>,
    #[serde(rename = "minItems")]
    pub min_items: Option<usize>,
    #[serde(rename = "maxItems")]
    pub max_items: Option<usize>,
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,
    #[serde(rename = "minContains")]
    pub min_contains: Option<usize>,
    #[serde(rename = "maxContains")]
    pub max_contains: Option<usize>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TagItems {
    #[serde(rename = "type")]
    pub tag_type: ItemType,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    pub minimum: Option<usize>,
    pub maximum: Option<usize>,
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<usize>,
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<usize>,
    #[serde(rename = "minLength")]
    pub min_length: Option<usize>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<usize>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TagContains {
    #[serde(rename = "type")]
    pub tag_type: ItemType,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    pub minimum: Option<usize>,
    pub maximum: Option<usize>,
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<usize>,
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<usize>,
    #[serde(rename = "minLength")]
    pub min_length: Option<usize>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<usize>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryDescriptor {
    #[serde(rename = "message_timestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: Option<QueryFilter>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct QueryFilter {
    pub protocol: Option<String>,
    pub recipient: Option<DIDBuf>,
}
