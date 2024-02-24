use serde_wasm_bindgen::Serializer;

#[inline]
pub(crate) fn serializer() -> Serializer {
    Serializer::new().serialize_maps_as_objects(true)
}
