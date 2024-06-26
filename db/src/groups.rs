use std::fmt::Debug;

use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use abi::model::group::Group;
#[async_trait::async_trait(?Send)]
pub trait GroupInterface: Debug {
    async fn put(&self, group: &Group) -> Result<(), JsValue>;

    async fn get(&self, id: &str) -> Result<Option<Group>, JsValue>;

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Group>, JsValue>;

    async fn delete(&self, id: &str) -> Result<(), JsValue>;

    async fn dismiss(&self, id: &str) -> Result<(), JsValue>;
}
