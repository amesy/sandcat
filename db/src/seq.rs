use std::fmt::Debug;

use wasm_bindgen::JsValue;

use abi::model::seq::Seq;

/// seq's id is always 1
#[async_trait::async_trait(?Send)]
pub trait SeqInterface: Debug {
    async fn put(&self, seq: &Seq) -> Result<(), JsValue>;

    async fn get(&self) -> Result<Seq, JsValue>;
}
