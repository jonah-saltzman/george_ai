use axum::{
    extract::{Json, State},
    routing::get,
    Router
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use dotenv;
use openai::{chat::{ChatCompletionMessage, ChatCompletionMessageRole as Role, ChatCompletion}, set_key};
use std::{collections::HashMap, sync::Arc};

struct AppState {
    conversations: HashMap<String, Vec<ChatCompletionMessage>>,
}

#[derive(Deserialize, Serialize)]
struct NewMessage {
    user: String,
    msg: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    set_key(std::env::var("OPENAI_KEY").unwrap());
    let app_state = Arc::new(Mutex::new(AppState {
        conversations: HashMap::new(),
    }));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }).post(post_chat))
        .with_state(app_state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn post_chat(State(state): State<Arc<Mutex<AppState>>>, body: Json<NewMessage>) -> Json<NewMessage> {
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
        let ret = Json(NewMessage { user: body.user.clone(), msg: returned_message.content.as_ref().unwrap().clone() });
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
        let first_msg = ChatCompletionMessage { role: Role::User, content: Some(body.msg.clone()), name: None, function_call: None };
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
        let ret = Json(NewMessage { user: body.user.clone(), msg: returned_message.content.as_ref().unwrap().clone() });
        messages.push(returned_message);
        state.conversations.insert(body.user.clone(), messages);
        ret
    }
}

fn print_msg(msg: &ChatCompletionMessage) {
    let role: &str = match msg.role {
        Role::User => "User",
        Role::Assistant => "AI",
        _ => { return }
    };
    println!("{}: {}", role, msg.content.as_ref().unwrap());
}