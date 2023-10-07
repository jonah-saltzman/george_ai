use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{function_component, html, use_mut_ref, use_state};
use gloo_net::http::Request;
use shared::NewMessage;
use crate::web_println;

#[function_component(Chat)]
pub fn chat() -> Html {
    let messages = use_state(|| vec![]);
    let input = use_mut_ref(|| "".to_string());

    let send_message = {
        let input_str = input.borrow().clone();
        // *input.borrow_mut() = "".to_string();
        let messages = messages.clone();
        let messages_fut = messages.clone();
        let messages_outer = messages.clone();
        use_callback(move |_, _| {
            let entered_message = NewMessage {user: "Jonah".to_owned(), msg: input_str.clone()};
            let entered_message_outer = entered_message.clone();
            web_println!("cb called, input_ref='{}'", input_str);
            let messages = messages_fut.clone();
            wasm_bindgen_futures::spawn_local(async move {
                web_println!("inside future, msg='{}'", entered_message.msg);
                let r = Request::post("/chat")
                    .header("Content-Type", "application/json")
                    .json(&entered_message)
                    .expect("Could not build request.")
                    .send()
                    .await
                    .unwrap()
                    .json::<NewMessage>()
                    .await;
                match r {
                    Ok(r) => {
                        let mut new_messages = (*messages).clone();
                        new_messages.push(r);
                        messages.set(new_messages);
                    },
                    Err(e) => {
                        web_println!("response: {}", e.to_string());
                    }
                }
            });

            let mut new_messages = (*messages_outer).clone();
            new_messages.push(entered_message_outer);
            messages_outer.set(new_messages);
        }, messages)
    };

    let on_input = {
        let input = input.clone();
        use_callback(move |e: InputEvent, _| {
            if let Some(input_value) = e.target_dyn_into::<HtmlInputElement>() {
                *input.borrow_mut() = input_value.value();
            } else {
                web_println!("not an input")
            }
        }, ())
    };

    html! {
        <div class="container text-center">
            <div class="messages">
                { for messages.iter().map(|message| html! { <p>{ message }</p> }) }
            </div>
            <div class="input_area">
                <input
                    type="text"
                    value={(*input).borrow().clone()}
                    oninput={on_input}
                    class="border-black"
                />
                <button onclick={send_message}>{"Send"}</button>
            </div>
        </div>
    }
}