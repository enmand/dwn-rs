use aes::cipher::generic_array::GenericArray;
use aes_gcm::{
    aead::{AeadMutInPlace, Buffer},
    Aes256Gcm, KeyInit,
};
use bytes::{Bytes, BytesMut};
use thiserror::Error;

use super::{Encryption, IVEncryption};

pub(super) struct AESBuffer<'a>(pub(crate) &'a mut BytesMut);

impl<'a> Buffer for AESBuffer<'a> {
    fn extend_from_slice(&mut self, other: &[u8]) -> aes_gcm::aead::Result<()> {
        self.0.extend_from_slice(other);

        Ok(())
    }

    fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

impl<'a> AsRef<[u8]> for AESBuffer<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'a> AsMut<[u8]> for AESBuffer<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

pub struct AES256GCM {
    cipher: Aes256Gcm,
    iv: Option<GenericArray<u8, typenum::consts::U12>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("AES-256-GCM encryption/decryption error: {0}")]
    EncryptError(aes_gcm::Error),
    #[error("AES-256-GCM Initialization Vector error")]
    NoIVError,
}

impl Encryption for AES256GCM {
    fn new(key: &[u8; 32]) -> Result<Self, super::Error> {
        let cipher = Aes256Gcm::new(key.into());
        Ok(Self { cipher, iv: None })
    }

    fn encrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, super::Error> {
        let mut data = AESBuffer(data);
        if let Some(iv) = &self.iv {
            self.cipher
                .encrypt_in_place(iv, b"", &mut data)
                .map_err(Error::EncryptError)?;
            Ok(data.0.clone().freeze())
        } else {
            Err(Error::NoIVError.into())
        }
    }

    fn decrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, super::Error> {
        let mut data = AESBuffer(data);
        if let Some(iv) = &self.iv {
            self.cipher
                .decrypt_in_place(iv, b"", &mut data)
                .map_err(Error::EncryptError)?;
            Ok(data.0.clone().freeze())
        } else {
            Err(Error::NoIVError.into())
        }
    }
}

impl IVEncryption for AES256GCM {
    type NonceSize = typenum::consts::U12;

    fn with_iv(&mut self, iv: GenericArray<u8, Self::NonceSize>) -> Result<Self, super::Error> {
        Ok(Self {
            cipher: self.cipher.clone(),
            iv: Some(iv),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const KEY: [u8; 32] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c, 0x76, 0x3b, 0x61, 0x7b, 0x2e, 0x45, 0x8f, 0x17, 0x98, 0x4a, 0xc3, 0x5b, 0x4d, 0xa4,
        0x5c, 0x2a,
    ];
    const IV: [u8; 12] = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb,
    ];

    #[test]
    fn test_aes256gcm() {
        let mut enc = AES256GCM::new(&KEY)
            .unwrap()
            .with_iv(IV.into())
            .expect("IV error");

        let data = BytesMut::from("Hello, world!");

        let enc_data = enc.encrypt(&mut data.clone()).unwrap();
        let dec_data = enc.decrypt(&mut enc_data.clone().into()).unwrap();

        assert_ne!(data, enc_data);
        assert_eq!(data, dec_data);
    }

    #[test]
    fn test_aes256gcm_no_iv() {
        let mut enc = AES256GCM::new(&KEY).unwrap();

        let data = BytesMut::from("Hello, world!");

        let enc_data = enc.encrypt(&mut data.clone());
        let dec_data = enc.decrypt(&mut data.clone());

        assert!(enc_data.is_err());
        assert!(dec_data.is_err());
    }
}
