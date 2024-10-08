use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{MapValue, Pagination};

/// ReadDescriptor represents the RecordsRead interface method for reading a given
/// record by ID.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

// QueryDescriptor represents the RecordsQuery interface method for querying records.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct QueryDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters,
    pub pagination: Option<Pagination>,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<DateSort>,
}

/// DataSort represents Records ordering for queries.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum DateSort {
    #[serde(rename = "createdAscending")]
    CreatedAscending,
    #[serde(rename = "createdDescending")]
    CreatedDescending,
    #[serde(rename = "publishedAscending")]
    PublishedAscending,
    #[serde(rename = "publishedDescending")]
    PublishedDescending,
}

/// WriteDescriptor represents the RecordsWrite interface method for writing a record to the DWN.
/// It can be represented with either no additional fields (`()`), or additional descriptor fields,
/// as in the case for `encodedData`.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct WriteDescriptor {
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    pub recipient: Option<String>,
    pub schema: Option<String>,
    pub tags: Option<MapValue>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "dataCid")]
    pub data_cid: String,
    #[serde(rename = "dataSize")]
    pub data_size: u64,
    #[serde(
        rename = "dateCreated",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub date_created: chrono::DateTime<chrono::Utc>,
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub published: Option<bool>,
    #[serde(
        rename = "datePublished",
        serialize_with = "crate::ser::serialize_optional_datetime"
    )]
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "dataFormat")]
    pub data_format: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub prune: bool,
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::{DateTime, SecondsFormat, Utc};

    use super::*;

    #[test]
    fn test_read_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let rd = ReadDescriptor {
            message_timestamp,
            record_id: "test".to_string(),
        };

        let ser = serde_json::to_string(&rd).unwrap();
        let de: ReadDescriptor = serde_json::from_str(&ser).unwrap();

        assert_eq!(rd, de);
    }

    #[test]
    fn test_query_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let qd = QueryDescriptor {
            message_timestamp,
            filter: Default::default(),
            pagination: None,
            date_sort: None,
        };

        let ser = serde_json::to_string(&qd).unwrap();
        let de: QueryDescriptor = serde_json::from_str(&ser).unwrap();

        assert_eq!(qd, de);
    }

    #[test]
    fn test_write_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let wd = WriteDescriptor {
            protocol: None,
            protocol_path: None,
            recipient: None,
            schema: None,
            tags: None,
            parent_id: None,
            data_cid: "test".to_string(),
            data_size: 0,
            date_created: message_timestamp,
            message_timestamp,
            published: None,
            date_published: None,
            data_format: "test".to_string(),
        };

        let ser = serde_json::to_string(&wd).unwrap();
        let de: WriteDescriptor = serde_json::from_str(&ser).unwrap();

        assert_eq!(wd, de);
    }

    #[test]
    fn test_subscribe_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let sd = SubscribeDescriptor {
            message_timestamp,
            filter: Default::default(),
        };

        let ser = serde_json::to_string(&sd).unwrap();
        let de: SubscribeDescriptor = serde_json::from_str(&ser).unwrap();

        assert_eq!(sd, de);
    }

    #[test]
    fn test_delete_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let dd = DeleteDescriptor {
            message_timestamp,
            record_id: "test".to_string(),
            prune: false,
        };

        let ser = serde_json::to_string(&dd).unwrap();
        let de: DeleteDescriptor = serde_json::from_str(&ser).unwrap();

        assert_eq!(dd, de);
    }
}
