use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use xtra::{Address, Mailbox};

use crate::{
    descriptors::MessageDescriptor,
    errors::{EventStreamError, StoreError},
    MapValue, Message,
};
use tracing::{instrument, trace};

use super::{Emit, EventChannel, EventStream, MessageEvent, Shutdown, Subscribe, Subscription};

#[derive(Debug, Default)]
pub struct EventStreamer<D>(Option<Address<EventStream<D>>>)
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static;

impl<D> EventStreamer<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static,
{
    pub fn new() -> Self {
        Self(None)
    }

    #[cfg(target_arch = "wasm32")]
    #[instrument]
    pub async fn open(&mut self) {
        trace!("opening EventStreamer (wasm)");
        self.0 = Some(xtra::spawn_wasm_bindgen(
            EventStream::new(),
            Mailbox::unbounded(),
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[instrument]
    pub async fn open(&mut self) {
        trace!("opening EventStreamer (tokio)");
        self.0 = Some(xtra::spawn_tokio(EventStream::new(), Mailbox::unbounded()));
    }

    #[instrument]
    pub async fn close(&mut self) {
        if let Some(addr) = self.0.take() {
            let _ = addr.send(Shutdown).await;
        }
    }

    #[instrument]
    pub async fn emit(&self, ns: &str, evt: MessageEvent<D>, indexes: MapValue) {
        if let Some(addr) = &self.0 {
            let _ = addr
                .send(Emit {
                    ns: ns.to_string(),
                    evt,
                    indexes,
                })
                .await;
        }
    }

    #[instrument]
    pub async fn subscribe(
        &self,
        ns: &str,
        id: &str,
        listener: EventChannel<D>,
    ) -> Result<Subscription, EventStreamError> {
        if let Some(addr) = &self.0 {
            trace!("subscribing to event stream");
            let sub = addr
                .send(Subscribe {
                    ns: ns.to_string(),
                    id: id.to_string(),
                    listener,
                })
                .await?;
            return Ok(sub);
        }

        Err(EventStreamError::StoreError(StoreError::NoInitError))
    }
}
