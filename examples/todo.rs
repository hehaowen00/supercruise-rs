use supercruise_rs::common::Dir;
use supercruise_rs::prelude::*;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

static TODOS: Lazy<Mutex<HashMap<u64, Todo>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static ID_COUNTER: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::from(0));

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    id: u64,
    text: String,
    status: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    id: Option<u64>,
    text: Option<String>,
    status: Option<bool>,
}

impl Todo {
    pub fn new(id: u64, text: String) -> Self {
        Self {
            id,
            text,
            status: false,
        }
    }
}

async fn index(req: Request<Body>) -> std::io::Result<Response<Body>> {
    let mut f = tokio::fs::File::open("index.html").await?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await?;

    let resp: Response<Body> = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Connection", "close")
        .body(buf.clone().into())
        .unwrap();

    Ok(resp)
}

async fn get_todos(req: Request<Body>) -> std::io::Result<Response<Body>> {
    let items = TODOS.lock().await;
    let json = serde_json::to_string(&*items).unwrap();
    let resp: Response<Body> = Response::builder()
        .status(StatusCode::OK)
        .body(json.into())
        .unwrap();

    Ok(resp)
}

async fn post_todo(req: Request<Body>) -> std::io::Result<Response<Body>> {
    let id = ID_COUNTER.fetch_add(1, Ordering::AcqRel);

    let mut items = TODOS.lock().await;
    items.insert(id, Todo::new(id, format!("Task {}", id)));

    let resp = Response::builder()
        .status(StatusCode::OK)
        .body(String::from("ok").into())
        .unwrap();

    Ok(resp)
}

async fn update_todo() {
    let mut items = TODOS.lock().await;
}

async fn delete_todo() {
    let mut items = TODOS.lock().await;
}

fn main() {
    let router = Router::new()
        .get("/static/*", Dir::new("static"))
        .get("/", index)
        .get("/todos", get_todos)
        .post("/todos", post_todo);

    // supercruise_rs::serve(router).await
    supercruise_rs::start_server("0.0.0.0:8080", router);
}
