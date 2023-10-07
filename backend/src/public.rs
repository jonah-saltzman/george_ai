use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response, Redirect},
    Router,
    routing::get
};
use http::{header, StatusCode};
use std::{collections::HashMap, sync::Arc};
use tokio::{fs, sync::RwLock};
use tokio::io::Result;
use mime_guess;
use tracing::{trace, debug};

type PublicFiles = HashMap<String, Vec<u8>>;


pub struct StaticFile {
    data: Vec<u8>,
    name: String
}

impl IntoResponse for StaticFile {
    fn into_response(self) -> Response {
        let content_type = mime_guess::from_path(self.name).first_or_text_plain().to_string();
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            self.data,
        )
            .into_response()
    }
}

pub enum FileResponse {
    File(StaticFile),
    Redirect(Redirect)
}

impl IntoResponse for FileResponse {
    fn into_response(self) -> Response {
        match self {
            FileResponse::File(f) => f.into_response(),
            FileResponse::Redirect(r) => r.into_response()
        }
    }
}

pub async fn get_static_files(path: &str) -> Result<PublicFiles> {
    let mut files = HashMap::new();
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if let Some(filename) = path.file_name() {
            let contents = fs::read(&path).await?;
            files.insert(filename.to_str().unwrap().to_string(), contents);
        }
    }
    Ok(files)
}

pub async fn root_handler(State(state): State<Arc<RwLock<PublicFiles>>>) -> StaticFile {
    let state = state.read().await;
    let data = state.get("index.html").expect("there to be an index.html");
    StaticFile { data: data.clone(), name: "index.html".to_string() }
}

pub async fn file_handler(
    State(state): State<Arc<RwLock<PublicFiles>>>,
    Path(name): Path<String>,
) -> FileResponse {
    trace!(file = name);
    let state = state.read().await;
    match state.get(&name) {
        Some(data) => FileResponse::File(StaticFile { data: data.clone(), name: name.to_string() }),
        None => {
            debug!("requested file missing: {}", name);
            FileResponse::Redirect(Redirect::temporary("/"))
        }
    }
}

pub async fn get_public_router() -> Router {
    let files = get_static_files(&std::env::var("PUBLIC_PATH").unwrap()).await.expect("to find public files");
    let files = Arc::new(RwLock::new(files));
    Router::new()
        .route("/", get(root_handler))
        .route("/index.html", get(|| async { Redirect::temporary("/") }))
        .route("/:name", get(file_handler))
        .with_state(files)
}
