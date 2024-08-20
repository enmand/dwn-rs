use std::{collections::BTreeMap, future::Future, pin::Pin};

use futures_util::future;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, Instrument};
use xtra::{prelude::MessageChannel, Actor, Handler};

use crate::{errors::EventStreamError, MapValue, Message};

pub type Event = (String, MessageEvent, MapValue);

pub type EventChannel = MessageChannel<Event, MessageEvent, xtra::refcount::Strong>;

#[derive(Debug, Default)]
pub struct EventStream {
    listeners: BTreeMap<(String, String), EventChannel>,
}

impl EventStream {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct Emit {
    pub ns: String,
    pub evt: MessageEvent,
    pub indexes: MapValue,
}

#[derive(Debug)]
pub struct Subscribe {
    pub ns: String,
    pub id: String,
    pub listener: EventChannel,
}

#[derive(Debug, Clone)]
pub struct Close {
    pub ns: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Shutdown;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MessageEvent {
    message: Message,
    #[serde(rename = "initialWrite")]
    initial_write: Option<Message>, // RecordsWrite message
}

impl Actor for EventStream {
    type Stop = ();

    #[instrument]
    async fn stopped(self) {
        info!("EventStream stopped");
    }
}

impl Handler<Emit> for EventStream {
    type Return = ();

    async fn handle(&mut self, msg: Emit, _ctx: &mut xtra::Context<Self>) -> Self::Return {
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

impl Handler<Subscribe> for EventStream {
    type Return = Subscription;

    async fn handle(&mut self, msg: Subscribe, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        debug!("handling event subscription");
        let ns = msg.ns;
        let id = msg.id;
        let listener = msg.listener;
        let addr = _ctx.mailbox().address().try_upgrade().unwrap();

        let sub = Subscription {
            id: id.clone(),
            close: Box::new(make_close_task(ns.clone(), id.clone(), addr)),
        };

        self.listeners.insert((ns, id), listener);
        sub
    }
}

impl Handler<Close> for EventStream {
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

impl Handler<Shutdown> for EventStream {
    type Return = ();

    async fn handle(&mut self, _: Shutdown, _ctx: &mut xtra::Context<Self>) -> Self::Return {
        debug!("Shutting down EventStream");
        self.listeners.clear();
        _ctx.stop_all();
    }
}

#[allow(clippy::type_complexity)]
pub struct Subscription {
    pub id: String,
    pub close: Box<
        dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), EventStreamError>> + Send>>
            + Send
            + Sync,
    >,
}

#[allow(dead_code)]
#[instrument]
fn make_close_task(
    ns: String,
    id: String,
    addr: xtra::Address<EventStream>,
) -> impl Fn() -> Pin<Box<dyn Future<Output = Result<(), EventStreamError>> + Send>> + 'static {
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
    use tracing_test::traced_test;
    use xtra::{spawn_tokio, Mailbox};

    use crate::{descriptors::Records, Descriptor, Fields};

    #[traced_test]
    #[tokio::test]
    async fn test_event_stream() {
        use super::*;

        fn test_evt() -> MessageEvent {
            MessageEvent {
                message: Message {
                    descriptor: Descriptor::Records(Records::Read(Default::default())),
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

        struct MessageReturner(Option<MessageEvent>);
        impl Actor for MessageReturner {
            type Stop = Option<MessageEvent>;

            async fn stopped(self) -> Self::Stop {
                self.0
            }
        }

        impl Handler<Event> for MessageReturner {
            type Return = MessageEvent;

            async fn handle(
                &mut self,
                (ns, msg, indexes): (String, MessageEvent, MapValue),
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
        assert_eq!(sub.id, sub_id);

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
