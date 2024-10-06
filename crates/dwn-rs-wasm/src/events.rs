use alloc::boxed::Box;

use dwn_rs_core::{MessageEvent as CoreMessageEvent, Subscription};
use futures_util::FutureExt;
use js_sys::{Object, Promise, Reflect};
use serde::Serialize;
use wasm_bindgen::{prelude::*, throw_str};

#[wasm_bindgen(typescript_custom_section)]
const EVENT_IMPORT: &'static str =
    r#"import { MessageEvent, EventListener, EventSubscription } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "MessageEvent")]
    pub type MessageEvent;

    #[wasm_bindgen(typescript_type = "EventSubscription")]
    pub type EventSubscription;
}

impl From<&MessageEvent> for CoreMessageEvent {
    fn from(value: &MessageEvent) -> Self {
        if value.is_undefined() {
            throw_str("MessageEvent is undefined");
        }

        serde_wasm_bindgen::from_value(value.into()).expect_throw("unable to deserialize event")
    }
}

impl From<CoreMessageEvent> for MessageEvent {
    fn from(value: CoreMessageEvent) -> Self {
        value
            .serialize(&crate::ser::serializer())
            .expect_throw("unable to serialize event")
            .into()
    }
}

impl TryFrom<Subscription> for EventSubscription {
    type Error = JsValue;

    fn try_from(value: Subscription) -> Result<EventSubscription, JsValue> {
        let obj: EventSubscription = JsCast::unchecked_into(Object::new());
        Reflect::set(&obj, &"id".into(), &value.subscription_id.id.into())?;
        Reflect::set(
            &obj,
            &"close".into(),
            &Closure::once(Box::new(move || {
                wasm_bindgen_futures::future_to_promise((value.close)().map(
                    |r| -> Result<JsValue, JsValue> {
                        r.expect_throw("unable to close subscription");
                        Ok(JsValue::UNDEFINED)
                    },
                ))
            }) as Box<dyn Fn() -> Promise>)
            .into_js_value(),
        )?;

        Ok(obj)
    }
}

impl core::fmt::Debug for MessageEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MessageEvent").finish()
    }
}
