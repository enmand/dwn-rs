use crate::auth::Authorization;
use crate::descriptors::MessageDescriptor;
use crate::filters::message_filters::Messages as MessagesFilter;
use crate::interfaces::messages::descriptors::{MESSAGES, QUERY, READ, SUBSCRIBE};
use cid::Cid;
use dwn_rs_message_derive::descriptor;
use serde::{Deserialize, Serialize};

use super::{MessageParameters, MessageValidator};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct ReadParameters {
    #[serde(rename = "messageCid")]
    pub message_cid: Cid,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
}

impl MessageValidator for ReadParameters {}

impl MessageParameters for ReadParameters {
    type Descriptor = ReadDescriptor;
    type Fields = Authorization;
}

#[descriptor(interface = MESSAGES, method = READ, fields = crate::auth::Authorization, parameters = ReadParameters)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "messageCid", skip_serializing_if = "Option::is_none")]
    pub message_cid: Option<Cid>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct QueryParameters {
    pub filters: Option<MessagesFilter>,
    pub cursor: Option<crate::Cursor>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
}

impl MessageValidator for QueryParameters {}

impl MessageParameters for QueryParameters {
    type Descriptor = QueryDescriptor;
    type Fields = Authorization;
}

#[descriptor(interface = MESSAGES, method = QUERY, fields = crate::auth::Authorization, parameters = QueryParameters)]
pub struct QueryDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::Filters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<crate::Cursor>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct SubscribeParameters {
    pub filters: Option<MessagesFilter>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
}

impl MessageValidator for SubscribeParameters {}

impl MessageParameters for SubscribeParameters {
    type Descriptor = SubscribeDescriptor;
    type Fields = Authorization;
}

#[descriptor(interface = MESSAGES, method = SUBSCRIBE, fields = crate::auth::Authorization, parameters = SubscribeParameters)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::Filters>,
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use chrono::{DateTime, SecondsFormat, Utc};
    use serde_json::json;

    #[test]
    fn test_read_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        // new random DagCbor encoded CID
        let message_cid = Cid::new_v1(0x71, cid::multihash::Multihash::default());
        let descriptor = ReadDescriptor {
            message_timestamp,
            message_cid: Some(message_cid),
        };
        let json = json!({
            "messageTimestamp": message_timestamp,
            "messageCid": message_cid,
            "interface": MESSAGES,
            "method": READ,
        });
        assert_eq!(serde_json::to_value(&descriptor).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<ReadDescriptor>(json).unwrap(),
            descriptor
        );
    }

    #[test]
    fn test_query_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let filters = vec![crate::Filters::default()];
        let cursor = Some(crate::Cursor::default());
        let descriptor = QueryDescriptor {
            message_timestamp,
            filters,
            cursor: cursor.clone(),
        };
        let json = json!({
            "messageTimestamp": message_timestamp,
            "filters": [crate::Filters::default()],
            "cursor": cursor,
            "interface": MESSAGES,
            "method": QUERY,
        });
        assert_eq!(serde_json::to_value(&descriptor).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<QueryDescriptor>(json).unwrap(),
            descriptor
        );
    }

    #[test]
    fn test_subscribe_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let filters = vec![crate::Filters::default()];
        let descriptor = SubscribeDescriptor {
            message_timestamp,
            filters,
        };
        let json = json!({
            "messageTimestamp": message_timestamp,
            "filters": [crate::Filters::default()],
            "interface": MESSAGES,
            "method": SUBSCRIBE
        });
        assert_eq!(serde_json::to_value(&descriptor).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<SubscribeDescriptor>(json).unwrap(),
            descriptor
        );
    }
}
