pub mod cid;

use k256::{PublicKey, SecretKey};
use partially::Partial;
use rand::{distributions::Alphanumeric, Rng};
use ssi_dids_core::DIDBuf;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum URLError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Invalid URL scheme: {0}")]
    InvalidScheme(String),
}

pub fn normalize_url(url: &str) -> Result<String, URLError> {
    let mut url = Url::parse(url)?;

    url.set_fragment(None);
    url.set_query(None);

    if url.path().is_empty() {
        url.set_path("/");
    }

    if url.scheme().is_empty() {
        let _ = url.set_scheme("http");
    }

    Ok(url.to_string())
}

#[derive(Error, Debug)]
pub enum PersonaError {
    #[error("DID error: {0}")]
    DIDError(#[from] ssi_dids_core::InvalidDID<String>),
}

#[derive(Partial, Debug)]
#[partially(derive(Default))]
pub struct Persona {
    pub did: DIDBuf,
    pub key_id: String,
    keypair: (SecretKey, PublicKey),
}

impl Persona {
    pub fn generate(
        PartialPersona {
            did,
            key_id,
            keypair,
        }: PartialPersona,
    ) -> Result<Self, PersonaError> {
        let did = did.unwrap_or_else(|| {
            let suffix = generate_random_string(32);
            DIDBuf::from_str(&format!("did:example:{}", suffix)).unwrap()
        });

        let keypair = keypair.unwrap_or_else(|| {
            let rng = &mut rand::thread_rng();
            let secp = SecretKey::random(rng);

            (secp.clone(), secp.public_key())
        });

        let key_id = key_id.unwrap_or_else(|| {
            let suffix = generate_random_string(16);
            format!("{}#{}", did, suffix)
        });

        Ok(Self {
            did,
            key_id,
            keypair,
        })
    }

    pub fn public_key(&self) -> k256::PublicKey {
        self.keypair.1
    }
}

pub fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
