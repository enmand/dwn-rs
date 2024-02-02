use dwn_rs_stores::surrealdb::SurrealDB;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = SurrealDataStore)]
pub struct SurrealDataStore {
    store: SurrealDB,
}

#[wasm_bindgen(js_class = SurrealDataStore)]
impl SurrealDataStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            store: SurrealDB::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsValue> {
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {}
}
