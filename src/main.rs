mod remote_session;
mod remote_machine;
mod monitor;
mod handlers;

use std::{
    env,
    sync::Arc
};

use axum::{
    routing::{get, post},
    Router,
    Extension,
};

use crate::monitor::Monitor;

const ADDR_PORT: &str = "0.0.0.0:5000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();

    let mut monitor = Monitor::new(args.get(1)
        .expect("Missing argument - Configuration path file not fount"));
    monitor.setup().await;
    let monitor_state = Arc::new(monitor);

    let app = Router::new()
        .route("/api/v1/users", get(handlers::get_users))
        .route("/api/v1/file", get(handlers::get_remotes))
        .route("/api/v1/file", post(handlers::post_file))
        .layer(Extension(monitor_state));

    let listener = tokio::net::TcpListener::bind(ADDR_PORT).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
