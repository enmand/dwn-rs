use aes::cipher::generic_array::GenericArray;
use aes_gcm::{aead::AeadMutInPlace, KeyInit};
use bytes::{Bytes, BytesMut};
use crypto_secretbox::XSalsa20Poly1305 as XSalsa20Poly1305Cipher;
use thiserror::Error;

use super::{aes_gcm::AESBuffer, Encryption, IVEncryption};

pub struct XSalsa20Poly1305 {
    cipher: XSalsa20Poly1305Cipher,
    iv: Option<GenericArray<u8, typenum::consts::U24>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("XSalsa20Poly1305 encryption/decryption error: {0}")]
    EncryptError(crypto_secretbox::Error),
    #[error("XSalsa20Poly1305 Initialization Vector error")]
    NoIVError,
}

impl Encryption for XSalsa20Poly1305 {
    fn new(key: &[u8; 32]) -> Result<Self, super::Error> {
        let cipher = XSalsa20Poly1305Cipher::new(key.into());
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

impl IVEncryption for XSalsa20Poly1305 {
    type NonceSize = typenum::consts::U24;

    fn with_iv(&mut self, iv: GenericArray<u8, Self::NonceSize>) -> Result<Self, super::Error> {
        Ok(Self {
            cipher: self.cipher.clone(),
            iv: Some(iv),
        })
    }
}
