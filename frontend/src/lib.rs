pub mod app;
pub mod components;

use serde::de::DeserializeOwned;

#[macro_export]
macro_rules! web_println {
    ($($arg:tt)*) => {
        {
            use web_sys::console;
            let formatted_string = format!($($arg)*);
            let js_value = wasm_bindgen::JsValue::from_str(&formatted_string);
            console::log_1(&js_value);
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    RequestError,
    DeserializeError,
    // etc.
}

pub async fn fetch<T>(url: String) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let response = reqwest::get(url).await;
    if let Ok(data) = response {
        if let Ok(repo) = data.json::<T>().await {
            Ok(repo)
        } else {
            Err(Error::DeserializeError)
        }
    } else {
        Err(Error::RequestError)
    }
}