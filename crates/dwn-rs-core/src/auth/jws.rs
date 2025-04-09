use base64::prelude::{BASE64_URL_SAFE_NO_PAD as base64url, *};
use cid::Cid;
use futures_util::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use ssi_claims_core::SignatureError;
use ssi_jws::{JwsPayload, JwsSigner};
use thiserror::Error;

use crate::MapValue;

#[derive(Error, Debug)]
pub enum JwsError {
    #[error("Error parsing JWS: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Error signing JWS: {0}")]
    SignError(#[from] ssi_claims_core::SignatureError),
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct JWS {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<SignatureEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<MapValue>,
    #[serde(flatten)] // TODO: remove?
    pub extra: MapValue,
}

#[derive(Serialize)]
pub struct Payload {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: Cid,
    #[serde(rename = "delegatedGrantId", skip_serializing_if = "Option::is_none")]
    pub delegated_grant_id: Option<Cid>,
    #[serde(rename = "permissionGrantId", skip_serializing_if = "Option::is_none")]
    pub permission_grant_id: Option<String>,
    #[serde(rename = "protocolRule", skip_serializing_if = "Option::is_none")]
    pub protocol_rule: Option<String>,
}

impl JwsPayload for Payload {
    fn payload_bytes(&self) -> std::borrow::Cow<[u8]> {
        let payload = serde_json::to_vec(self).expect("could not serialize payload");
        std::borrow::Cow::Owned(payload)
    }
}

impl JWS {
    pub async fn create<S, P>(payload: P, signers: Option<Vec<S>>) -> Result<Self, JwsError>
    where
        S: JwsSigner,
        P: JwsPayload,
    {
        let encoded_payload = base64url.encode(payload.payload_bytes());

        if let Some(signers) = signers {
            let signatures = Self::generate_signatures(signers, &payload).await?;
            Ok(Self {
                payload: Some(encoded_payload),
                signatures: Some(signatures),
                header: None,
                extra: MapValue::default(),
            })
        } else {
            Err(JwsError::SignError(SignatureError::MissingSigner))
        }
    }

    async fn generate_signatures<S, P>(
        signers: Vec<S>,
        payload: P,
    ) -> Result<Vec<SignatureEntry>, JwsError>
    where
        S: JwsSigner,
        P: JwsPayload + Clone + Copy,
    {
        stream::iter(signers)
            .then(|signer| async move {
                let payload = payload.clone();
                let result: Result<SignatureEntry, JwsError> = async {
                    let payload = payload.clone();
                    let signature = signer.sign_into_decoded(payload).await?;

                    Ok(SignatureEntry {
                        protected: Some(signature.header().encode()),
                        signature: Some(signature.signature.encode()),
                        extra: MapValue::default(),
                    })
                }
                .await;

                result
            })
            .try_collect()
            .await
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct SignatureEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(flatten)] // TODO: remove?
    pub extra: MapValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssi_jwk::JWK;

    #[tokio::test]
    async fn test_jws_create() {
        let jwk = JWK::generate_secp256k1();
        let jws = JWS::create(b"hello world".to_vec(), Some(vec![jwk]))
            .await
            .expect("could not create JWS");

        assert_eq!(jws.payload, Some("aGVsbG8gd29ybGQ".to_string()));
        assert_eq!(jws.signatures.as_ref().unwrap().len(), 1);
        assert_eq!(
            jws.signatures.as_ref().unwrap()[0]
                .protected
                .as_ref()
                .unwrap(),
            "eyJhbGciOiJFUzI1NksifQ"
        );

        assert!(!jws.signatures.as_ref().unwrap()[0]
            .signature
            .as_ref()
            .unwrap()
            .is_empty());
    }
}
