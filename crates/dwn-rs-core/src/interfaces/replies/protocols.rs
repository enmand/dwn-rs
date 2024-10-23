use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{descriptors::protocols, Message};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Query {
    pub entries: Option<Message<protocols::ConfigureDescriptor>>,
}
