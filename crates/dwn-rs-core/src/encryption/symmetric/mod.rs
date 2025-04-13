use std::{pin::Pin, task::Poll};

use aes::cipher::{generic_array::GenericArray, ArrayLength};
use bytes::{Bytes, BytesMut};
use futures_util::{ready, Stream};
use pin_project_lite::pin_project;
use thiserror::Error;

pub mod aead;
pub mod aes_ctr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("AES-256-CTR encryption error: {0}")]
    AES256CTR(#[from] aes_ctr::Error),
    #[error("AEAD encryption error: {0}")]
    AEAD(#[from] aead::Error),
}

impl<T: ?Sized> StreamEncryptionExt for T where T: Stream {}

pub trait StreamEncryptionExt: Stream {
    fn encrypt<E>(self, key: GenericArray<u8, E::KeySize>) -> Result<Encrypt<Self, E>, Error>
    where
        E: Encryption,
        E::KeySize: ArrayLength<u8>,
        Self: Sized,
    {
        Encrypt::new(self, key)
    }

    fn decrypt<E>(self, key: GenericArray<u8, E::KeySize>) -> Result<Decrypt<Self, E>, Error>
    where
        E: Encryption,
        E::KeySize: ArrayLength<u8>,
        Self: Sized,
    {
        Decrypt::new(self, key)
    }
}

pub trait Encryption {
    type KeySize: ArrayLength<u8>;

    fn new(key: GenericArray<u8, Self::KeySize>) -> Result<Self, Error>
    where
        Self: Sized;
    fn encrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, Error>;
    fn decrypt(&mut self, data: &mut BytesMut) -> Result<Bytes, Error>;
}

pub trait IVEncryption: Encryption {
    type NonceSize: ArrayLength<u8>;

    fn with_iv(&mut self, iv: GenericArray<u8, Self::NonceSize>) -> Result<Self, Error>
    where
        Self: Sized;

    fn nonce(&self) -> GenericArray<u8, Self::NonceSize> {
        let mut nonce = GenericArray::default();
        nonce.iter_mut().for_each(|b| *b = rand::random());
        nonce
    }
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
    E::KeySize: ArrayLength<u8>,
{
    pub fn new(stream: D, key: GenericArray<u8, E::KeySize>) -> Result<Self, Error> {
        Ok(Self {
            stream,
            encryption: E::new(key)?,
        })
    }
}

impl<D, E> Encrypt<D, E>
where
    E: IVEncryption,
{
    pub fn with_iv(mut self, iv: GenericArray<u8, E::NonceSize>) -> Result<Self, Error> {
        self.encryption = self.encryption.with_iv(iv)?;
        Ok(self)
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
    pub fn new(stream: D, key: GenericArray<u8, E::KeySize>) -> Result<Self, Error> {
        Ok(Self {
            stream,
            encryption: E::new(key)?,
        })
    }
}

impl<D, E> Decrypt<D, E>
where
    E: IVEncryption,
{
    pub fn with_iv(mut self, iv: GenericArray<u8, E::NonceSize>) -> Result<Self, Error> {
        self.encryption = self.encryption.with_iv(iv)?;
        Ok(self)
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
