use dwn_rs_core::{
    emitter::EventStreamer, subscription::SubscriptionFn, MapValue,
    MessageEvent as CoreMessageEvent,
};
use js_sys::Promise;
use tokio::sync::mpsc::unbounded_channel;
use tracing::{instrument, trace};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use crate::{
    events::{EventSubscription, MessageEvent},
    filter::IndexMap,
};

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct EventStream {
    events: EventStreamer,
}

#[wasm_bindgen]
impl EventStream {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self::default()
    }

    #[wasm_bindgen]
    pub async fn open(&mut self) {
        trace!("opening event stream from wasm");
        self.events.open().await;
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        trace!("closing event stream from wasm");
        self.events.close().await;
    }

    #[wasm_bindgen]
    pub async fn emit(&self, tenant: &str, evt: &MessageEvent, indexes: IndexMap) {
        trace!("emitting event from wasm");
        let indextags = indexes.into();
        self.events.emit(tenant.into(), evt.into(), indextags).await;
    }

    #[wasm_bindgen]
    pub async fn subscribe(
        &self,
        tenant: &str,
        id: String,
        listener: js_sys::Function,
    ) -> Result<EventSubscription, JsError> {
        trace!("subscribing js function to event stream");
        let sub = subscription_for_func(id.clone(), listener).await?.run();

        self.events
            .subscribe(tenant.into(), id, SubscriptionFn::channel(sub))
            .await
            .map_err(JsError::from)
            .map(|s| s.try_into().expect_throw("unable to convert subscription"))
    }
}

#[instrument]
async fn subscription_for_func(
    id: String,
    listener: js_sys::Function,
) -> Result<SubscriptionFn, JsError> {
    trace!("creating subscription for js function");
    let (tx, mut rx) = unbounded_channel::<(String, CoreMessageEvent, MapValue)>();

    spawn_local(async move {
        while let Some((tenant, evt, indexes)) = rx.recv().await {
            trace!(
                tenant = ?tenant,
                indexes = ?indexes,
                event = ?evt,
                "calling listener for event",
            );
            let tenant = JsValue::from_str(&tenant);
            let evt: MessageEvent = evt.into();
            let indexes: IndexMap = indexes.into();

            let prom: JsValue = listener
                .call3(&JsValue::NULL, &tenant, &evt.into(), &indexes)
                .expect_throw("unable to call listener function");

            if prom.is_instance_of::<Promise>() {
                let prom: Promise = prom.into();
                JsFuture::from(prom)
                    .await
                    .expect_throw("unable to resolve promise");
            }
        }
    });

    let subscription = SubscriptionFn::new(
        id,
        Box::new(move |tenant, evt, indexes| {
            trace!("sending event to listener");
            tx.send((tenant, evt, indexes)).unwrap_throw();
        }),
    );

    Ok(subscription)
}
