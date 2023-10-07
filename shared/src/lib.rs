use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use serde_wasm_bindgen::to_value;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct NewMessage {
    pub user: String,
    pub msg: String,
}

impl From<NewMessage> for JsValue {
    fn from(value: NewMessage) -> Self {
        to_value(&value).unwrap()
    }
}

impl std::fmt::Display for NewMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.user, self.msg))
    }
}