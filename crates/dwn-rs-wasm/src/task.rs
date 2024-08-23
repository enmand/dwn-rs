use core::fmt::Debug;

use alloc::vec::Vec;
use dwn_rs_core::{stores::ManagedResumableTask, value::Value};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::ser::serializer;

#[wasm_bindgen(typescript_custom_section)]
const TASK_IMPORT: &str = r"import { ManagedResumableTask } from '@tbd54566975/dwn-sdk-js';";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ManagedResumableTask")]
    pub type JsManagedResumableTask;

    #[wasm_bindgen(typescript_type = "ManagedResumableTask[]")]
    pub type JsManagedResumableTaskArray;
}

impl<T> From<ManagedResumableTask<T>> for JsManagedResumableTask
where
    T: Serialize + Sync + Send + Debug,
{
    fn from(task: ManagedResumableTask<T>) -> Self {
        task.serialize(&serializer())
            .expect_throw("unable to serialize task")
            .into()
    }
}

impl<T> From<&ManagedResumableTask<T>> for JsManagedResumableTask
where
    T: Serialize + Sync + Send + Debug,
{
    fn from(task: &ManagedResumableTask<T>) -> Self {
        serde_wasm_bindgen::to_value(task)
            .expect_throw("unable to serialize task")
            .dyn_into()
            .expect_throw("unable to convert task managed task ref")
    }
}

impl From<Vec<ManagedResumableTask<Value>>> for JsManagedResumableTaskArray {
    fn from(tasks: Vec<ManagedResumableTask<Value>>) -> Self {
        tasks
            .serialize(&serializer())
            .expect_throw("unable to serialize tasks")
            .into()
    }
}
