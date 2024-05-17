mod remote_session;
mod remote_machine;
mod monitor;

use std::{
    env,
    sync::Arc
};
use std::collections::HashMap;

use axum::{
    extract::Json,
    routing::{get, post},
    Router,
    Extension,
};
use serde::Deserialize;

use crate::monitor::Monitor;

const ADDR_PORT: &str = "0.0.0.0:5000";

async fn get_users(monitor_state: Extension<Arc<Monitor>>) -> String {
    let mut users = Vec::new();

    for machine in &monitor_state.config.remotes {
       users.push(&machine.usr);
    }

    serde_json::to_string(&users).unwrap()
}

async fn get_remotes(monitor_state: Extension<Arc<Monitor>>) -> String {
    let mut files = HashMap::new();

    for machine in &monitor_state.config.remotes {
        files.insert(
            &machine.usr,
            machine.read_file_data().await.expect("Cannnot obtain machine data")
        );
    }

    serde_json::to_string(&files).unwrap()
}

#[derive(Deserialize)]
struct PostFile {
    user: String,
    file_ctx: String
}

async fn post_file(monitor_state: Extension<Arc<Monitor>>, Json(query): Json<PostFile>) {
    if let Some(machine) = &monitor_state.get_machine_by_name(&query.user) {
        machine.write_file(&query.file_ctx).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();

    let mut monitor = Monitor::new(args.get(1)
        .expect("Missing argument - Configuration path file not fount"));
    monitor.setup().await;
    let monitor_state = Arc::new(monitor);

    let app = Router::new()
        .route("/users", get(get_users))
        .route("/files", get(get_remotes))
        .route("/file", post(post_file))
        .layer(Extension(monitor_state));

    let listener = tokio::net::TcpListener::bind(ADDR_PORT).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
