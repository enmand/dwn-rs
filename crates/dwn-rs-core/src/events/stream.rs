use std::{collections::BTreeMap, fmt::Debug, future::Future, pin::Pin};

use futures_util::future;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, Instrument};
use xtra::{prelude::MessageChannel, Actor, Handler};

use crate::{
    descriptors::{records, MessageDescriptor},
    errors::EventStreamError,
    Descriptor, MapValue, Message,
};

pub type Event<D> = (String, MessageEvent<D>, MapValue);

pub type EventChannel<D> = MessageChannel<Event<D>, MessageEvent<D>, xtra::refcount::Strong>;

#[derive(Debug)]
pub struct EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    listeners: BTreeMap<(String, String), EventChannel<D>>,
}

impl<D> EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    pub fn new() -> Self {
        EventStream {
            listeners: BTreeMap::new(),
        }
    }
}

impl<D> Default for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Emit<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    pub ns: String,
    pub evt: MessageEvent<D>,
    pub indexes: MapValue,
}

#[derive(Debug)]
pub struct Subscribe<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    pub ns: String,
    pub id: String,
    pub listener: EventChannel<D>,
}

#[derive(Debug, Clone)]
pub struct Close {
    pub ns: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Shutdown;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct MessageEvent<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    pub message: Message<D>,
    #[serde(rename = "initialWrite")]
    pub initial_write: Option<Message<records::WriteDescriptor>>,
}

// This is a custom deserializer for the MessageEvent struct. It is necessary because the
// Message struct has a generic type parameter that is not known at compile time. This deserializer
// is the generalized version, which can deserialize any descriptor type. Individual
// Descriptors types implement their own deserializers via. the `MessageDescriptor` trait
// derivation.
impl<'de> Deserialize<'de> for MessageEvent<Descriptor> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempEvent {
            message: Message<Descriptor>,
            #[serde(rename = "initialWrite")]
            initial_write: Option<Message<records::WriteDescriptor>>,
        }

        let temp_event = TempEvent::deserialize(deserializer)?;

        Ok(Self {
            message: temp_event.message,
            initial_write: temp_event.initial_write,
        })
    }
}

impl<D> Actor for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    type Stop = ();

    #[instrument]
    async fn stopped(self) {
        info!("EventStream stopped");
    }
}

impl<D> Handler<Emit<D>> for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    type Return = ();

    async fn handle(&mut self, msg: Emit<D>, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        debug!("Emitting event");
        future::join_all(
            self.listeners
                .iter()
                .filter_map(
                    |((listener_ns, _), listener)| match listener_ns == &msg.ns {
                        true => Some(listener),
                        false => None,
                    },
                )
                .map(|listener| {
                    let ns = msg.ns.clone();
                    let evt = msg.evt.clone();
                    let indexes = msg.indexes.clone();
                    async move {
                        trace!(
                            ns = ?ns,
                            listener = ?listener.clone(),
                            "sending event to listener",
                        );
                        listener
                            .send((ns.clone(), evt.clone(), indexes.clone()))
                            .await
                            .expect("Failed to send message");
                    }
                }),
        )
        .instrument(tracing::debug_span!("emit"))
        .await;
    }
}

impl<D> Handler<Subscribe<D>> for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    type Return = Subscription;

    async fn handle(&mut self, msg: Subscribe<D>, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        debug!("handling event subscription");
        let ns = msg.ns;
        let id = msg.id;
        let listener = msg.listener;
        let addr = _ctx.mailbox().address().try_upgrade().unwrap();

        let sub = Subscription {
            subscription_id: SubscriptionID { id: id.clone() },
            close: Box::new(make_close_task(ns.clone(), id.clone(), addr)),
        };

        self.listeners.insert((ns, id), listener);
        sub
    }
}

impl<D> Handler<Close> for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    type Return = ();

    async fn handle(&mut self, close: Close, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        let listener = self.listeners.remove(&(close.ns.clone(), close.id.clone()));
        debug!(
            target = "Closing EventStream subscription",
            ns = close.ns,
            id = close.id,
            listener = listener.is_some(),
        );
    }
}

impl<D> Handler<Shutdown> for EventStream<D>
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    type Return = ();

    async fn handle(&mut self, _: Shutdown, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        debug!("Shutting down EventStream");
        self.listeners.clear();
        _ctx.stop_all();
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SubscriptionID {
    pub id: String,
}

#[allow(clippy::type_complexity)]
pub struct Subscription {
    pub subscription_id: SubscriptionID,
    pub close: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), EventStreamError>> + Send>>
            + Send
            + Sync,
    >,
}

#[allow(dead_code)]
#[instrument]
fn make_close_task<D>(
    ns: String,
    id: String,
    addr: xtra::Address<EventStream<D>>,
) -> impl Fn() -> Pin<Box<dyn Future<Output = Result<(), EventStreamError>> + Send>> + 'static
where
    Message<D>: Serialize + DeserializeOwned,
    D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
{
    move || {
        let ns = ns.clone();
        let id = id.clone();

        let close = Close { ns, id };
        let addr = addr.clone();

        let fut = async move {
            trace!("closing event subscription task");
            addr.clone().send(close).await.unwrap();
            Ok(())
        };

        Box::pin(fut.instrument(tracing::info_span!("make_close_task")))
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;
    use tracing_test::traced_test;
    use xtra::{spawn_tokio, Mailbox};

    use crate::{descriptors::Records, Fields};

    #[traced_test]
    #[tokio::test]
    async fn test_event_stream() {
        use super::*;

        fn test_evt() -> MessageEvent<Descriptor> {
            let now = chrono::DateTime::<chrono::Utc>::MIN_UTC.naive_utc();
            MessageEvent {
                message: Message {
                    descriptor: Descriptor::Records(Records::Read(records::ReadDescriptor {
                        message_timestamp: chrono::DateTime::from_naive_utc_and_offset(
                            now,
                            chrono::Utc,
                        ),
                        filter: Default::default(),
                    })),
                    fields: Fields::Authorization(Default::default()),
                },
                initial_write: None,
            }
        }
        fn test_ns() -> String {
            "ns".to_string()
        }
        fn test_indexes() -> MapValue {
            MapValue::default()
        }

        struct MessageReturner<
            D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
        >(Option<MessageEvent<D>>)
        where
            Message<D>: Serialize + DeserializeOwned;
        impl<D> Actor for MessageReturner<D>
        where
            Message<D>: Serialize + DeserializeOwned,
            D: MessageDescriptor + DeserializeOwned + Clone + Debug + PartialEq + Send + 'static,
        {
            type Stop = Option<MessageEvent<D>>;

            async fn stopped(self) -> Self::Stop {
                self.0
            }
        }

        impl Handler<Event<Descriptor>> for MessageReturner<Descriptor> {
            type Return = MessageEvent<Descriptor>;

            async fn handle(
                &mut self,
                (ns, msg, indexes): (String, MessageEvent<Descriptor>, MapValue),
                _ctx: &mut xtra::Context<Self>,
            ) -> Self::Return {
                self.0 = Some(msg.clone());
                _ctx.stop_self();
                trace!("MessageReturner handling event");

                assert_eq!(msg, test_evt());
                assert_eq!(ns, test_ns());
                assert_eq!(indexes, test_indexes());

                msg
            }
        }

        let addr = spawn_tokio(EventStream::new(), Mailbox::unbounded());
        assert!(addr.is_connected());

        let (child_addr, child_mailbox) = Mailbox::unbounded();
        let child_fut = xtra::run(child_mailbox, MessageReturner(None));
        let child = tokio::spawn(child_fut);
        assert!(child_addr.is_connected());

        let sub_id = "test";
        let sub = addr
            .send(Subscribe {
                ns: test_ns(),
                id: sub_id.to_string(),
                listener: MessageChannel::new(child_addr),
            })
            .instrument(tracing::info_span!("subscribe"))
            .await
            .unwrap();
        assert_eq!(sub.subscription_id.id, sub_id);

        let emit = addr
            .send(Emit {
                ns: test_ns(),
                evt: test_evt(),
                indexes: test_indexes(),
            })
            .await;
        assert!(emit.is_ok());

        let f = (sub.close)().await;
        assert!(f.is_ok());

        let s = child.await;
        assert!(s.is_ok());
        let opt_msg = s.unwrap();
        assert!(opt_msg.is_some());
        assert_eq!(opt_msg.unwrap(), test_evt());
    }
}
