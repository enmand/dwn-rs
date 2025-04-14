use crate::auth::Authorization;
use crate::descriptors::{MessageDescriptor, ValidationError};
use crate::encryption::asymmetric::publickey::PublicKey;
use crate::encryption::asymmetric::PublicKeyTrait;
use crate::encryption::{DerivationScheme, KeyEncryptionAlgorithm};
use crate::fields::WriteFields;
use crate::filters::message_filters::Records as RecordsFilter;
use crate::interfaces::messages::descriptors::{DELETE, QUERY, READ, RECORDS, SUBSCRIBE, WRITE};
use crate::{normalize_url, MapValue, Message, Pagination};

use dwn_rs_message_derive::descriptor;

use super::{MessageParameters, MessageValidator};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_jwk::JWK;
use ssi_jws::JwsSigner;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
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

impl MessageValidator for ReadParameters {
    fn validate(&self) -> Result<(), super::ValidationError> {
        Ok(())
    }
}

impl MessageParameters for ReadParameters {
    type Descriptor = ReadDescriptor;
    type Fields = crate::auth::Authorization;

    async fn build<S: JwsSigner>(
        &self,
        signer: Option<S>,
    ) -> Result<(Self::Descriptor, self::Authorization), super::ValidationError> {
        let descriptor = ReadDescriptor {
            message_timestamp: self.message_timestamp.unwrap_or_else(chrono::Utc::now),
            filter: self.filters.clone(),
        };

        let mut authorization = Authorization::default();
        if let Some(signer) = signer {
            authorization = Message::create_authorization(
                &descriptor,
                signer,
                self.delegated_grant.clone(),
                self.permission_grant_id.clone(),
                self.protocol_role.clone(),
            )
            .await?;
        }

        Ok((descriptor, authorization))
    }
}

/// ReadDescriptor represents the RecordsRead interface method for reading a given
/// record by ID.
#[descriptor(interface = RECORDS, method = READ, fields = crate::auth::Authorization, parameters = ReadParameters)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: RecordsFilter,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
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

impl MessageValidator for QueryParameters {
    fn validate(&self) -> Result<(), super::ValidationError> {
        if let Some(filter) = self.filter.clone() {
            if let Some(published) = filter.published {
                if let Some(date_sort) = self.date_sort.clone() {
                    if date_sort == DateSort::PublishedAscending && !published {
                        return Err(super::ValidationError {
                            message: "Cannot sort by publish date when published is false"
                                .to_string(),
                        });
                    }

                    if date_sort == DateSort::PublishedDescending && !published {
                        return Err(super::ValidationError {
                            message: "Cannot sort by publish date when published is false"
                                .to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

impl MessageParameters for QueryParameters {
    type Descriptor = QueryDescriptor;
    type Fields = Authorization;

    async fn build<S: JwsSigner>(
        &self,
        signer: Option<S>,
    ) -> Result<(Self::Descriptor, Self::Fields), super::ValidationError> {
        let descriptor = QueryDescriptor {
            message_timestamp: self.message_timestamp.unwrap_or_else(chrono::Utc::now),
            filter: self.filter.clone().unwrap_or_default(),
            date_sort: self.date_sort.clone(),
            pagination: self.pagination.clone(),
        };

        let mut authorization = Authorization::default();
        if let Some(signer) = signer {
            authorization = Message::create_authorization(
                &descriptor,
                signer,
                self.delegated_grant.clone(),
                None,
                self.protocol_role.clone(),
            )
            .await?;
        }

        Ok((descriptor, authorization))
    }
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
    pub filter: RecordsFilter,
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
    pub key_encryption_input: Vec<KeyEncryptionInput>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct KeyEncryptionInput {
    #[serde(rename = "derivationSchema")]
    pub derivation_schema: DerivationScheme,
    #[serde(rename = "publicKeyId")]
    pub public_key_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: JWK,
    pub algorithm: Option<KeyEncryptionAlgorithm>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
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
    pub data_cid: Option<String>,
    #[serde(rename = "dataSize")]
    pub data_size: Option<u64>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
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

impl MessageValidator for WriteParameters {
    fn validate(&self) -> Result<(), super::ValidationError> {
        if self.protocol.is_none() && self.protocol_path.is_some()
            || self.protocol.is_some() && self.protocol_path.is_none()
        {
            return Err(super::ValidationError {
                message: "protocol and protocolPath must be either both set or both unset"
                    .to_string(),
            });
        }

        if self.data.is_none() && self.data_cid.is_none()
            || self.data.is_some() && self.data_cid.is_some()
        {
            return Err(super::ValidationError {
                message: "data and dataCid must be either both set or both unset".to_string(),
            });
        }

        if self.data.is_some() && self.data_size.is_none()
            || self.data.is_none() && self.data_size.is_some()
        {
            return Err(super::ValidationError {
                message: "data and dataSize must be either both set or both unset".to_string(),
            });
        }

        if let Some(encryption_input) = &self.encryption_input {
            encryption_input.key_encryption_input.iter().map(|input| {
                match (&input.derivation_schema, &self.protocol, &self.schema) {
                    (DerivationScheme::ProtocolPath, None, _) => Err(ValidationError {
                        message: "'protocols' encryption requires a protocol".to_string(),
                    }),
                    (DerivationScheme::Schemas, _, None) => Err(ValidationError {
                        message: "'schemas' encryption requires a schema".to_string(),
                    }),
                    (_, Some(_), Some(_)) => Ok(()),
                    (_, _, _) => Ok(()),
                }
            });
        }

        Ok(())
    }
}

impl MessageParameters for WriteParameters {
    type Descriptor = WriteDescriptor;
    type Fields = WriteFields;

    async fn build<S: JwsSigner>(
        &self,
        _signer: Option<S>,
    ) -> Result<(Self::Descriptor, Self::Fields), super::ValidationError> {
        let data_cid = self.data_cid.clone().unwrap_or(
            crate::cid::generate_cid(self.data.clone().unwrap_or_default())
                .map_err(|e| ValidationError {
                    message: e.to_string(),
                })?
                .to_string(),
        );
        let data_size = self.data_size.unwrap_or_else(|| {
            self.data
                .as_ref()
                .map(|data| data.len() as u64)
                .unwrap_or_default()
        });

        let now = chrono::Utc::now();

        let mut descriptor = WriteDescriptor {
            protocol: self
                .protocol
                .clone()
                .and_then(|url| normalize_url(&url).ok()),
            protocol_path: self.protocol_path.clone(),
            recipient: self.recipient.clone(),
            schema: self.schema.clone().and_then(|url| normalize_url(&url).ok()),
            tags: self.tags.clone(),
            parent_id: self.parent_context_id.clone().and_then(|context_id| {
                Some(
                    context_id
                        .split("/")
                        .filter(|segment| !segment.is_empty())
                        .last()?
                        .to_string(),
                )
            }),
            data_cid,
            data_size,
            date_created: self.date_created.unwrap_or(now),
            message_timestamp: self.message_timestamp.unwrap_or(now),
            published: self.published,
            date_published: self.date_published,
            data_format: self.data_format.clone(),
        };

        if let (Some(published), None) = (self.published, self.date_published) {
            if published {
                descriptor.date_published = Some(now);
            }
        }

        if let Some(encryption_input) = &self.encryption_input {
            let key_encryption = encryption_input.key_encryption_input.iter().map(
                |input| -> Result<(), crate::encryption::asymmetric::Error> {
                    let jwk = PublicKey::try_from(input.public_key.to_public())?;
                    let key_enc_output = jwk.encrypt(&encryption_input.key)?;

                    Ok(())
                },
            );
        }

        todo!()
    }
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct SubscribeParameters {
    pub filters: RecordsFilter,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "protocolRole")]
    pub protocol_role: Option<String>,
    #[serde(rename = "delegatedGrant")]
    pub delegated_grant: Option<WriteFields>,
}

impl MessageParameters for SubscribeParameters {
    type Descriptor = SubscribeDescriptor;
    type Fields = Authorization;
}

#[descriptor(interface = RECORDS, method = SUBSCRIBE, fields = crate::auth::Authorization, parameters = SubscribeParameters)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
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

impl MessageParameters for DeleteParameters {
    type Descriptor = DeleteDescriptor;
    type Fields = Authorization;
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

    #[tokio::test]
    async fn test_read_descriptor() {
        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        )
        .unwrap();

        let rp = ReadParameters {
            message_timestamp: Some(message_timestamp),
            filters: RecordsFilter::default(),
            ..Default::default()
        };

        let rd = ReadDescriptor {
            message_timestamp,
            filter: RecordsFilter::default(),
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
