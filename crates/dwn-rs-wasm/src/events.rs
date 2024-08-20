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
        match serde_wasm_bindgen::from_value(value.into()) {
            Ok(m) => m,
            Err(e) => throw_str(&format!("unable to deserialize event: {:?}", e)),
        }
    }
}

impl From<CoreMessageEvent> for MessageEvent {
    fn from(value: CoreMessageEvent) -> Self {
        match value.serialize(&crate::ser::serializer()) {
            Ok(m) => m.into(),
            Err(e) => throw_str(&format!("unable to serialize event: {:?}", e)),
        }
    }
}

impl TryFrom<Subscription> for EventSubscription {
    type Error = JsValue;

    fn try_from(value: Subscription) -> Result<EventSubscription, JsValue> {
        let obj: EventSubscription = JsCast::unchecked_into(Object::new());
        Reflect::set(&obj, &"id".into(), &value.id.into())?;
        Reflect::set(
            &obj,
            &"close".into(),
            &Closure::once(Box::new(move || {
                wasm_bindgen_futures::future_to_promise((value.close)().map(
                    |r| -> Result<JsValue, JsValue> {
                        match r {
                            Ok(_) => Ok(JsValue::UNDEFINED),
                            Err(e) => throw_str(&format!("{:?}", e)),
                        }
                    },
                ))
            }) as Box<dyn Fn() -> Promise>)
            .into_js_value(),
        )?;

        Ok(obj)
    }
}

impl std::fmt::Debug for MessageEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageEvent").finish()
    }
}
