use serde::{Deserialize, Serialize};

use crate::{descriptors::records::WriteDescriptor, fields::MessageFields, Message};

use super::jws::JWS;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Authorization {
    pub signature: JWS,
    #[serde(
        rename = "authorDelegatedGrant",
        skip_serializing_if = "Option::is_none"
    )]
    pub author_delegated_grant: Option<Box<Message<WriteDescriptor>>>,
    #[serde(rename = "ownerSignature", skip_serializing_if = "Option::is_none")]
    pub owner_signature: Option<JWS>,
    #[serde(
        rename = "ownerDelegatedGrant",
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_delegated_grant: Option<Box<Message<WriteDescriptor>>>,
}

impl MessageFields for Authorization {}
