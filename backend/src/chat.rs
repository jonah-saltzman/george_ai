use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole as Role};
use shared::NewMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, trace};

pub struct ChatState {
    conversations: HashMap<String, Vec<ChatCompletionMessage>>,
}

async fn post_chat(
    State(state): State<Arc<Mutex<ChatState>>>,
    body: Json<NewMessage>,
) -> Json<NewMessage> {
    let mut state = state.lock().await;
    trace!(chat = body.msg);

    // Check if user already exists in the conversations
    let user_exists = state.conversations.contains_key(&body.user);

    // If not, initialize with the system message
    if !user_exists {
        debug!("new chat");
        let init_msg = ChatCompletionMessage {
            role: Role::System,
            content: Some("You are George Washington, the first president of the United States. You are being interviewed by a young student learning history. You answer with humor and lots of context. Do not reference the fact that you are an AI; the student should be immersed in the experience.".to_string()),
            name: None,
            function_call: None,
        };
        state
            .conversations
            .insert(body.user.clone(), vec![init_msg]);
    }

    // At this point, the user is guaranteed to exist in conversations, so we can safely get_mut
    let hist = state.conversations.get_mut(&body.user).unwrap();

    // Add the new message from user to the history
    let new_msg = ChatCompletionMessage {
        role: Role::User,
        content: Some(body.msg.clone()),
        name: None,
        function_call: None,
    };
    print_msg(&new_msg);
    hist.push(new_msg);

    // Interact with OpenAI API
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
    hist.push(returned_message.clone());

    // Return the response
    Json(NewMessage {
        user: body.user.clone(),
        msg: returned_message.content.as_ref().unwrap().clone(),
    })
}

fn print_msg(msg: &ChatCompletionMessage) {
    let role: &str = match msg.role {
        Role::User => "User",
        Role::Assistant => "AI",
        _ => return,
    };
    trace!("{}: {}", role, msg.content.as_ref().unwrap());
}

pub async fn get_chat_router() -> Router {
    let state = Arc::new(Mutex::new(ChatState {
        conversations: HashMap::new(),
    }));
    Router::new()
        .route("/chat", post(post_chat))
        .with_state(state)
}
