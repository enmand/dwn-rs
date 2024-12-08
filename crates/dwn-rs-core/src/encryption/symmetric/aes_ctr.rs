use aes::{
    cipher::{generic_array::GenericArray, KeyIvInit, StreamCipher, StreamCipherSeek},
    Aes256,
};
use bytes::{Bytes, BytesMut};
use ctr::Ctr64BE;
use thiserror::Error;

use super::{Encryption, IVEncryption};

pub type CipherAES256CTR = Ctr64BE<Aes256>;

pub struct AES256CTR {
    key: [u8; 32],
    enc: Option<CipherAES256CTR>,
    dec: Option<CipherAES256CTR>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid key length: {0}")]
    InvalidKeyLength(#[from] aes::cipher::InvalidLength),
    #[error("AES-256-CTR encryption/decryption error: {0}")]
    EncryptError(aes::cipher::StreamCipherError),
    #[error("AES-256-CTR IV error")]
    NoIVError,
}

impl Encryption for AES256CTR {
    fn new(key: &[u8; 32]) -> Result<Self, super::Error> {
        Ok(Self {
            key: *key,
            enc: None,
            dec: None,
        })
    }

    fn encrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, super::Error> {
        if let Some(enc) = &mut self.enc {
            enc.apply_keystream(data);
            Ok(data.clone().freeze())
        } else {
            Err(Error::NoIVError.into())
        }
    }

    fn decrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, super::Error> {
        if let Some(dec) = &mut self.dec {
            dec.apply_keystream(data);
            Ok(data.clone().freeze())
        } else {
            Err(Error::NoIVError.into())
        }
    }
}

impl IVEncryption for AES256CTR {
    type NonceSize = typenum::consts::U16;

    fn with_iv(&mut self, iv: GenericArray<u8, Self::NonceSize>) -> Result<Self, super::Error> {
        let cipher =
            CipherAES256CTR::new_from_slices(&self.key, &iv).map_err(Error::InvalidKeyLength)?;
        let mut dec_cipher = cipher.clone();
        dec_cipher.seek(0u32);

        Ok(Self {
            key: self.key,
            enc: Some(cipher),
            dec: Some(dec_cipher),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::Bytes;

    const KEY: [u8; 32] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c, 0x76, 0x3b, 0x61, 0x7b, 0x2e, 0x45, 0x8f, 0x17, 0x98, 0x4a, 0xc3, 0x5b, 0x4d, 0xa4,
        0x5c, 0x2a,
    ];
    const IV: [u8; 16] = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
        0xff,
    ];

    #[test]
    fn test_aes256ctr() {
        let mut enc = AES256CTR::new(&KEY)
            .expect("Failed to create AES256CTR")
            .with_iv(IV.into())
            .expect("Failed to set IV");

        let data = Bytes::from_static(b"hello world! this is my plaintext.");
        let enc_data = enc
            .encrypt(&mut data.clone().into())
            .unwrap_or_else(|e| panic!("{}", e.to_string()));

        assert_eq!(enc_data.to_vec(), b"\xde\xf2\xc6\xe6t\xec#x\x80\xce\xdb\xb1\x940\xa2\x0c\xab0\xef\0\x05B\"\x1eE\x92\xa6\xa4\xbe\x8c\x8dk\x5f\xDD");
        let dec_data = enc.decrypt(&mut enc_data.into()).unwrap();
        assert_eq!(data, dec_data);
    }

    #[test]
    fn test_aes256ctr_no_iv() {
        let mut enc = AES256CTR::new(&KEY).expect("Failed to create AES256CTR");

        let data = Bytes::from_static(b"hello world! this is my plaintext.");
        let enc_data = enc.encrypt(&mut data.clone().into());
        assert_eq!(
            enc_data.unwrap_err().to_string(),
            "AES-256-CBC encryption error: AES-256-CTR IV error"
        );

        let dec_data = enc.decrypt(&mut data.clone().into());
        assert_eq!(
            dec_data.unwrap_err().to_string(),
            "AES-256-CBC encryption error: AES-256-CTR IV error"
        );
    }
}
