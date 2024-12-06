use std::{pin::Pin, task::Poll};

use bytes::{Bytes, BytesMut};
use futures_util::{ready, Stream};
use pin_project_lite::pin_project;
use thiserror::Error;

pub mod aes_ctr;
pub mod aes_gcm;
pub mod xsalsa20_poly1305;

#[derive(Debug, Error)]
pub enum Error {
    #[error("AES-256-CBC encryption error: {0}")]
    AES256CTR(#[from] aes_ctr::Error),
}

impl<T: ?Sized> StreamEncryptionExt for T where T: Stream {}

pub trait StreamEncryptionExt: Stream {
    fn encrypt<E>(self, key: &[u8; 32], iv: &[u8; 16]) -> Result<Encrypt<Self, E>, Error>
    where
        E: Encryption,
        Self: Sized,
    {
        Encrypt::new(self, key, iv)
    }

    fn decrypt<E>(self, key: &[u8; 32], iv: &[u8; 16]) -> Result<Decrypt<Self, E>, Error>
    where
        E: Encryption,
        Self: Sized,
    {
        Decrypt::new(self, key, iv)
    }
}

pub trait Encryption {
    fn new(key: &[u8; 32], iv: &[u8; 16]) -> Result<Self, Error>
    where
        Self: Sized;
    fn encrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, Error>;
    fn decrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, Error>;
}

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Encrypt<D, E> {
        #[pin]
        stream: D,
        encryption: E,
    }
}

impl<D, E> Encrypt<D, E>
where
    E: Encryption,
{
    pub fn new(stream: D, key: &[u8; 32], iv: &[u8; 16]) -> Result<Self, Error> {
        Ok(Self {
            stream,
            encryption: E::new(key, iv)?,
        })
    }
}

impl<D, E> Stream for Encrypt<D, E>
where
    D: Stream<Item = Result<Bytes, Error>>,
    E: Encryption,
{
    type Item = Result<Bytes, Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let res = ready!(this.stream.as_mut().poll_next(cx));
        let mut bytes = match res {
            Some(Ok(bytes)) => bytes.into(),
            Some(Err(e)) => return Poll::Ready(Some(Err(e))),
            None => return Poll::Ready(None),
        };
        Poll::Ready(Some(this.encryption.encrypt(&mut bytes)))
    }
}

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Decrypt<D, E> {
        #[pin]
        stream: D,
        encryption: E,
    }
}

impl<D, E> Decrypt<D, E>
where
    E: Encryption,
{
    pub fn new(stream: D, key: &[u8; 32], iv: &[u8; 16]) -> Result<Self, Error> {
        Ok(Self {
            stream,
            encryption: E::new(key, iv)?,
        })
    }
}

impl<D, E> Stream for Decrypt<D, E>
where
    D: Stream<Item = Result<Bytes, Error>>,
    E: Encryption,
{
    type Item = Result<Bytes, Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let res = ready!(this.stream.as_mut().poll_next(cx));
        let mut bytes = match res {
            Some(Ok(bytes)) => bytes.into(),
            Some(Err(e)) => return Poll::Ready(Some(Err(e))),
            None => return Poll::Ready(None),
        };

        Poll::Ready(Some(this.encryption.decrypt(&mut bytes)))
    }
}

#[cfg(test)]
mod test {
    use super::{aes_ctr, Encryption, Error, StreamEncryptionExt};
    use bytes::Bytes;
    use futures_util::{pin_mut, stream, StreamExt};

    const KEY: [u8; 32] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c, 0x76, 0x3b, 0x61, 0x7b, 0x2e, 0x45, 0x8f, 0x17, 0x98, 0x4a, 0xc3, 0x5b, 0x4d, 0xa4,
        0x5c, 0x2a,
    ];
    const IV: [u8; 16] = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
        0xff,
    ];

    #[tokio::test]
    async fn test_aes256ctr_encrypt_stream() {
        let msg_part_1 = Bytes::from_static(b"hello world! ");
        let msg_part_2 = Bytes::from_static(b"this is my plaintext.");
        let msg = Bytes::from([msg_part_1.clone(), msg_part_2.clone()].concat());

        // Stream Encryption
        let data_stream = stream::iter(vec![
            Ok::<Bytes, Error>(msg_part_1.clone()),
            Ok(msg_part_2.clone()),
        ])
        .encrypt::<aes_ctr::AES256CTR>(&KEY, &IV)
        .expect("unable to generate encryption");

        // Static encryption
        let mut enc = aes_ctr::AES256CTR::new(&KEY, &IV).expect("Failed to create AES256CTR");
        let enc_data = enc
            .encrypt(&mut msg.clone().into())
            .unwrap_or_else(|e| panic!("{}", e.to_string()));

        pin_mut!(data_stream);
        let mut data: Vec<u8> = vec![];
        while let Some(c) = data_stream.next().await {
            data.extend_from_slice(&c.unwrap());
        }
        let data = Bytes::from(data);

        // Assert stream encryption and static encryption are equal
        assert_eq!(data, enc_data);
        // Assert the stream, and the static test do not match the original data
        assert_ne!(data, msg);
    }

    #[tokio::test]
    async fn test_aes256ctr_decrypt_stream() {
        let msg_part_1 = Bytes::from_static(b"hello world! ");
        let msg_part_2 = Bytes::from_static(b"this is my plaintext.");
        let msg = Bytes::from([msg_part_1.clone(), msg_part_2.clone()].concat());

        // Stream Encryption
        let data_stream = stream::iter(vec![
            Ok::<Bytes, Error>(msg_part_1.clone()),
            Ok(msg_part_2.clone()),
        ])
        .encrypt::<aes_ctr::AES256CTR>(&KEY, &IV)
        .expect("unable to generate encryption")
        .decrypt::<aes_ctr::AES256CTR>(&KEY, &IV)
        .expect("unable to generate decryption");

        // Static encryption
        let mut enc = aes_ctr::AES256CTR::new(&KEY, &IV).expect("Failed to create AES256CTR");
        let enc_data = enc
            .encrypt(&mut msg.clone().into())
            .unwrap_or_else(|e| panic!("{}", e.to_string()));
        let dec_data = enc
            .decrypt(&mut enc_data.clone().into())
            .unwrap_or_else(|e| panic!("{}", e.to_string()));

        pin_mut!(data_stream);
        let mut data: Vec<u8> = vec![];
        while let Some(c) = data_stream.next().await {
            data.extend_from_slice(&c.unwrap());
        }
        let data = Bytes::from(data);

        // Assert the stream, and the static test match the decrypted data
        assert_eq!(data, dec_data);
        // Assert the stream, and the static test match the original data
        assert_eq!(data, msg);
    }

    #[tokio::test]
    async fn test_aes256ctr_encrypt_err() {
        let msg = Bytes::from_static(b"hello world! ");

        // Stream Encryption
        let data_stream = stream::iter(vec![
            Ok(msg.clone()),
            Err(Error::AES256CTR(aes_ctr::Error::InvalidKeyLength(
                aes::cipher::InvalidLength,
            ))),
        ])
        .encrypt::<aes_ctr::AES256CTR>(&KEY, &IV)
        .expect("unable to generate encryption");

        pin_mut!(data_stream);
        while let Some(c) = data_stream.next().await {
            match c {
                Ok(_) => {}
                Err(e) => {
                    assert_eq!(
                        e.to_string(),
                        "AES-256-CBC encryption error: Invalid key length: Invalid Length"
                    );
                }
            }
        }
    }
}
