use xtra::{Address, Mailbox};

use crate::{
    errors::{EventStreamError, StoreError},
    MapValue,
};
use tracing::{instrument, trace};

use super::{Emit, EventChannel, EventStream, MessageEvent, Shutdown, Subscribe, Subscription};

#[derive(Debug, Default)]
pub struct EventStreamer(Option<Address<EventStream>>);
impl EventStreamer {
    #[cfg(target_arch = "wasm32")]
    #[instrument]
    pub async fn open(&mut self) {
        trace!("opening EventStreamer (wasm)");
        self.0 = Some(xtra::spawn_wasm_bindgen(
            EventStream::default(),
            Mailbox::unbounded(),
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[instrument]
    pub async fn open(&mut self) {
        trace!("opening EventStreamer (tokio)");
        self.0 = Some(xtra::spawn_tokio(
            EventStream::default(),
            Mailbox::unbounded(),
        ));
    }

    #[instrument]
    pub async fn close(&mut self) {
        if let Some(addr) = self.0.take() {
            let _ = addr.send(Shutdown).await;
        }
    }

    #[instrument]
    pub async fn emit(&self, ns: String, evt: MessageEvent, indexes: MapValue) {
        if let Some(addr) = &self.0 {
            let _ = addr.send(Emit { ns, evt, indexes }).await;
        }
    }

    #[instrument]
    pub async fn subscribe(
        &self,
        ns: String,
        id: String,
        listener: EventChannel,
    ) -> Result<Subscription, EventStreamError> {
        if let Some(addr) = &self.0 {
            trace!("subscribing to event stream");
            let sub = addr.send(Subscribe { ns, id, listener }).await?;
            return Ok(sub);
        }

        Err(EventStreamError::StoreError(StoreError::NoInitError))
    }
}
