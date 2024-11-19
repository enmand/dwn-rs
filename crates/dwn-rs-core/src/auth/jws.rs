use base64::prelude::{BASE64_URL_SAFE_NO_PAD as base64url, *};
use futures_util::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use ssi_jws::{Header, JwsSigner};
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

pub struct NoSigner {}
impl JwsSigner for NoSigner {
    async fn fetch_info(&self) -> Result<ssi_jws::JwsSignerInfo, ssi_claims_core::SignatureError> {
        Ok(ssi_jws::JwsSignerInfo {
            key_id: None,
            algorithm: ssi_jwk::Algorithm::None,
        })
    }

    async fn sign_bytes(
        &self,
        _signing_bytes: &[u8],
    ) -> Result<Vec<u8>, ssi_claims_core::SignatureError> {
        Ok(Vec::new())
    }
}

impl JWS {
    pub async fn create<S>(payload: Vec<u8>, signers: Option<Vec<S>>) -> Result<Self, JwsError>
    where
        S: JwsSigner,
    {
        let payload = base64url.encode(payload);

        if let Some(signers) = signers {
            let signatures = Self::generate_signatures(signers, &payload).await?;
            Ok(Self {
                payload: Some(payload),
                signatures: Some(signatures),
                header: None,
                extra: MapValue::default(),
            })
        } else {
            Ok(Self {
                payload: Some(payload),
                signatures: None,
                header: None,
                extra: MapValue::default(),
            })
        }
    }

    async fn generate_signatures<S>(
        signers: Vec<S>,
        payload_encoded: &str,
    ) -> Result<Vec<SignatureEntry>, JwsError>
    where
        S: JwsSigner,
    {
        stream::iter(signers)
            .then(|signer| async move {
                let result: Result<SignatureEntry, JwsError> = async {
                    let info = signer.fetch_info().await?;
                    let header = Header {
                        algorithm: info.algorithm,
                        key_id: info.key_id,
                        ..Default::default()
                    };
                    let header = serde_json::to_vec(&header)?;
                    let protected_header = base64url.encode(header);

                    let sign_input = format!("{}.{}", protected_header, payload_encoded);

                    let signature = signer.sign(&sign_input).await?;
                    let signature = base64url.encode(signature.as_bytes());

                    Ok(SignatureEntry {
                        protected: Some(protected_header),
                        signature: Some(signature),
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
        let jwk = JWK::generate_ed25519().expect("could not generate key");
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
            "eyJhbGciOiJFZERTQSJ9"
        );

        assert!(!jws.signatures.as_ref().unwrap()[0]
            .signature
            .as_ref()
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn test_jws_create_no_signers() {
        let jws = JWS::create::<NoSigner>(b"hello world".to_vec(), None)
            .await
            .expect("could not create JWS");

        assert_eq!(jws.payload, Some("aGVsbG8gd29ybGQ".to_string()));
        assert!(jws.signatures.is_none());
    }
}
