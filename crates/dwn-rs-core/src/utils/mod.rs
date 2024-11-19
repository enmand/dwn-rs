pub mod cid;

use partially::Partial;
use rand::{distributions::Alphanumeric, Rng};
use secp256k1::{Keypair, Secp256k1};
use ssi_dids_core::DIDBuf;
use std::str::FromStr;
use thiserror::Error;

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
    keypair: secp256k1::Keypair,
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
            let secp = Secp256k1::new();

            Keypair::new(&secp, &mut rand::thread_rng())
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

    pub fn public_key(&self) -> secp256k1::PublicKey {
        self.keypair.public_key()
    }
}

pub fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
