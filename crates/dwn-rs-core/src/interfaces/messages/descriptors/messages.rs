use cid::Cid;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "messageCid", skip_serializing_if = "Option::is_none")]
    pub message_cid: Option<Cid>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
        });
        assert_eq!(serde_json::to_value(&descriptor).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<SubscribeDescriptor>(json).unwrap(),
            descriptor
        );
    }
}
