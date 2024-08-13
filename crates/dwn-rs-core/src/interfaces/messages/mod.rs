pub mod descriptors;
pub mod fields;

pub use descriptors::Descriptor;
pub use fields::Fields;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Message {
    pub descriptor: Descriptor,
    #[serde(flatten)]
    pub fields: Fields, // Fields should be an Enum representing possible fields<D-s>
}
