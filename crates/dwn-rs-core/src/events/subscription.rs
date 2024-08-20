use tracing::{info, instrument, trace};
use xtra::{Actor, Address, Handler};

use crate::MapValue;

use super::{Event, EventChannel, MessageEvent};

pub type HandleFn = fn(String, MessageEvent, MapValue);

pub type SubscriptionFnAddress = Address<SubscriptionFn>;

/// SubscriptionFn is an actor that subscribes to events and calls a function when an event is emitted.
/// This is useful for functions that need to be called when an event is emitted, such as from
/// the WASM world, or through WebSockets.
pub struct SubscriptionFn {
    id: String,
    f: Box<dyn Fn(String, MessageEvent, MapValue) + Send + Sync + 'static>,
}

impl Actor for SubscriptionFn {
    type Stop = ();

    async fn stopped(self) -> Self::Stop {
        info!(target = "SubscriptionFn stopped", id = self.id);
    }
}

impl Handler<Event> for SubscriptionFn {
    type Return = MessageEvent;

    async fn handle(&mut self, evt: Event, _: &mut xtra::Context<Self>) -> Self::Return {
        trace!("SubscriptionFn handling event");
        (self.f)(evt.0, evt.1.clone(), evt.2);

        evt.1
    }
}

impl SubscriptionFn {
    pub fn new(
        id: String,
        f: Box<dyn Fn(String, MessageEvent, MapValue) + Send + Sync + 'static>,
    ) -> Self {
        Self { id, f }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn run(self) -> Address<Self> {
        trace!("starting actor (wasm)");
        xtra::spawn_wasm_bindgen(self, xtra::Mailbox::unbounded())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(self) -> Address<Self> {
        trace!("starting actor (tokio)");
        xtra::spawn_tokio(self, xtra::Mailbox::unbounded())
    }

    #[instrument]
    pub fn channel(addr: Address<Self>) -> EventChannel {
        trace!("adding SubscriptionFn to EventChannel");
        EventChannel::new(addr)
    }
}
