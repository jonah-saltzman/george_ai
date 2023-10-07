use dotenv;
use openai::set_key;
use std::env;
use tracing_subscriber::{prelude::*, EnvFilter};

mod chat;
mod public;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    set_key(env::var("OPENAI_KEY").unwrap());

    start_tracing();

    let public_router = public::get_public_router().await;
    let chat_router = chat::get_chat_router().await;

    let app = public_router.merge(chat_router);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn start_tracing() {
    let stdout = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "backend=debug".into())) // Only record debug and above from your crate
        .with(stdout) // For the actual logging
        .init();
}
