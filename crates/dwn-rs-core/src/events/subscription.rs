use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};
use tracing::{info, instrument, trace};
use xtra::{Actor, Address, Handler};

use crate::{descriptors::MessageDescriptor, MapValue, Message};

use super::{Event, EventChannel, MessageEvent};

pub type HandleFn<D> = fn(String, MessageEvent<D>, MapValue);
pub type SubscriptionFnAddress<D> = Address<SubscriptionFn<D>>;
pub type BoxedSubscriptionFn<D> =
    Box<dyn Fn(String, MessageEvent<D>, MapValue) + Send + Sync + 'static>;

/// SubscriptionFn is an actor that subscribes to events and calls a function when an event is emitted.
/// This is useful for functions that need to be called when an event is emitted, such as from
/// the WASM world, or through WebSockets.
pub struct SubscriptionFn<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static,
{
    id: String,
    f: BoxedSubscriptionFn<D>,
}

impl<D> Actor for SubscriptionFn<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static,
{
    type Stop = ();

    async fn stopped(self) -> Self::Stop {
        info!(target = "SubscriptionFn stopped", id = self.id);
    }
}

impl<D> Handler<Event<D>> for SubscriptionFn<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static,
{
    type Return = MessageEvent<D>;

    async fn handle(&mut self, evt: Event<D>, _: &mut xtra::Context<Self>) -> Self::Return {
        trace!("SubscriptionFn handling event");
        (self.f)(evt.0, evt.1.clone(), evt.2);

        evt.1
    }
}

impl<D> SubscriptionFn<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + Clone + Debug + PartialEq + Send + 'static,
{
    pub fn new(id: &str, f: BoxedSubscriptionFn<D>) -> Self {
        Self {
            id: id.to_string(),
            f,
        }
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
    pub fn channel(addr: Address<Self>) -> EventChannel<D> {
        trace!("adding SubscriptionFn to EventChannel");
        EventChannel::new(addr)
    }
}
