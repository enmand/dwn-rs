use crate::descriptors::MessageDescriptor;
use crate::encryption::{DerivationScheme, KeyEncryptionAlgorithm};
use crate::fields::WriteFields;
use crate::filters::messages::Records as RecordsFilter;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_jwk::JWK;

use crate::interfaces::messages::descriptors::{DELETE, QUERY, READ, RECORDS, SUBSCRIBE, WRITE};
use crate::{MapValue, Message, Pagination};
use dwn_rs_message_derive::descriptor;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadParameters {
    pub filters: RecordsFilter,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<Message<WriteDescriptor>>,
}

/// ReadDescriptor represents the RecordsRead interface method for reading a given
/// record by ID.
#[descriptor(interface = RECORDS, method = READ, fields = crate::fields::AuthorizationDelegatedGrantFields, parameters = RecordsFilter)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryParameters {
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub filter: Option<RecordsFilter>,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<DateSort>,
    pub pagination: Option<Pagination>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<Message<WriteDescriptor>>,
}

// QueryDescriptor represents the RecordsQuery interface method for querying records.
#[skip_serializing_none]
#[descriptor(interface = RECORDS, method = QUERY, fields = crate::auth::Authorization, parameters = QueryParameters)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EncryptionInput {
    pub algorithm: Option<KeyEncryptionAlgorithm>,
    #[serde(rename = "initializationVector")]
    pub initialization_vector: Vec<u8>,
    key: Vec<u8>,
    #[serde(rename = "keyEncryptionInput")]
    pub key_encryption_input: Option<KeyEncryptionInput>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct KeyEncryptionInput {
    #[serde(rename = "derivationSchema")]
    pub derivation_schema: Option<DerivationScheme>,
    #[serde(rename = "publicKeyId")]
    pub public_key_id: Option<String>,
    #[serde(rename = "publicKey")]
    pub public_key: Option<JWK>,
    pub algorithm: Option<KeyEncryptionAlgorithm>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WriteParameters {
    pub recipient: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    pub schema: Option<String>,
    pub tags: Option<MapValue>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "parentContextId")]
    pub parent_context_id: Option<String>,
    pub data: Option<Vec<u8>>,
    #[serde(rename = "dataCid")]
    pub data_cid: String,
    #[serde(rename = "dataSize")]
    pub data_size: u64,
    #[serde(rename = "dateCreated")]
    pub date_created: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub published: Option<bool>,
    #[serde(rename = "datePublished")]
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    pub data_format: String,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<Message<WriteDescriptor>>,
    #[serde(rename = "encryptionInput")]
    pub encryption_input: Option<EncryptionInput>,
    #[serde(rename = "permissionGrantId")]
    pub permission_grant_id: Option<String>,
}

/// WriteDescriptor represents the RecordsWrite interface method for writing a record to the DWN.
/// It can be represented with either no additional fields (`()`), or additional descriptor fields,
/// as in the case for `encodedData`.
#[skip_serializing_none]
#[descriptor(interface = RECORDS, method = WRITE, fields = crate::fields::WriteFields, parameters = WriteParameters)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SubscribeParameters {
    pub filters: RecordsFilter,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<WriteFields>,
}

#[descriptor(interface = RECORDS, method = SUBSCRIBE, fields = crate::fields::AuthorizationDelegatedGrantFields, parameters = SubscribeParameters)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteParameters {
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    #[serde(rename = "protocolRole")]
    pub prune: Option<bool>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<WriteFields>,
}

#[descriptor(interface = RECORDS, method = DELETE, fields = crate::auth::Authorization, parameters = DeleteParameters)]
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
            filter: crate::Filters::default(),
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
