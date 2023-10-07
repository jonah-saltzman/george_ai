use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole as Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ChatState {
    conversations: HashMap<String, Vec<ChatCompletionMessage>>,
}

#[derive(Deserialize, Serialize)]
struct NewMessage {
    user: String,
    msg: String,
}

async fn post_chat(
    State(state): State<Arc<Mutex<ChatState>>>,
    body: Json<NewMessage>,
) -> Json<NewMessage> {
    let mut state = state.lock().await;
    if let Some(hist) = state.conversations.get_mut(&body.user) {
        print!("existing");
        let new_msg = ChatCompletionMessage {
            role: Role::User,
            content: Some(body.msg.clone()),
            name: None,
            function_call: None,
        };
        print_msg(&new_msg);
        hist.push(new_msg);
        let completion = ChatCompletion::builder("gpt-3.5-turbo", hist.clone())
            .max_tokens(150u64)
            .temperature(0.9)
            .top_p(1.0)
            .presence_penalty(0.6)
            .stop(vec!["\n".to_string()])
            .create()
            .await
            .unwrap();
        let returned_message = completion.choices.first().unwrap().message.clone();
        print_msg(&returned_message);
        let ret = Json(NewMessage {
            user: body.user.clone(),
            msg: returned_message.content.as_ref().unwrap().clone(),
        });
        hist.push(returned_message);
        ret
    } else {
        println!("new chat");
        let mut messages = vec![ChatCompletionMessage {
            role: Role::System,
            content: Some("You are George Washington, the first president of the United States. You are being interviewed by a young student learning history. You answer with humor and lots of context. Do not reference the fact that you are an AI; the student should be immersed in the experience.".to_string()),
            name: None,
            function_call: None,
        }];
        let first_msg = ChatCompletionMessage {
            role: Role::User,
            content: Some(body.msg.clone()),
            name: None,
            function_call: None,
        };
        print_msg(&first_msg);
        messages.push(first_msg);
        let completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .max_tokens(150u64)
            .temperature(0.9)
            .top_p(1.0)
            .presence_penalty(0.6)
            .stop(vec!["\n".to_string()])
            .create()
            .await
            .unwrap();
        let returned_message = completion.choices.first().unwrap().message.clone();
        print_msg(&returned_message);
        let ret = Json(NewMessage {
            user: body.user.clone(),
            msg: returned_message.content.as_ref().unwrap().clone(),
        });
        messages.push(returned_message);
        state.conversations.insert(body.user.clone(), messages);
        ret
    }
}

#[allow(dead_code)]
fn print_msg(msg: &ChatCompletionMessage) {
    let role: &str = match msg.role {
        Role::User => "User",
        Role::Assistant => "AI",
        _ => return,
    };
    println!("{}: {}", role, msg.content.as_ref().unwrap());
}

pub async fn get_chat_router() -> Router {
    let state = Arc::new(Mutex::new(ChatState {
        conversations: HashMap::new(),
    }));
    Router::new().route("/chat", post(post_chat)).with_state(state)
}
