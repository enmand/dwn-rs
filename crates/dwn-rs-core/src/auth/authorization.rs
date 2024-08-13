use serde::{Deserialize, Serialize};

use crate::Message;

use super::jws::JWS;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Authorization {
    pub signature: JWS,
    #[serde(rename = "ownerSignature", skip_serializing_if = "Option::is_none")]
    pub owner_signature: Option<JWS>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct AuthorizationDelegatedGrant {
    pub signature: JWS,
    #[serde(
        rename = "authorDelegatedGrant",
        skip_serializing_if = "Option::is_none"
    )]
    pub author_delegated_grant: Option<Box<Message>>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct AuthorizationOwner {
    pub signature: JWS,
    #[serde(
        rename = "authorDelegatedGrant",
        skip_serializing_if = "Option::is_none"
    )]
    pub author_delegated_grant: Option<Box<Message>>,
    #[serde(rename = "ownerSignature", skip_serializing_if = "Option::is_none")]
    pub owner_signature: Option<JWS>,
    #[serde(
        rename = "ownerDelegatedGrant",
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_delegated_grant: Option<Box<Message>>,
}
