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
    http::{HeaderValue, Method},
    Router,
    Extension,
};
use tower_http::cors::CorsLayer;

use crate::monitor::Monitor;

const ADDR_PORT: &str = "0.0.0.0:5000";
const CONF_PATH_ARG: usize = 1;
const ORIGIN_ARG: usize = 2;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();

    let mut monitor = Monitor::new(args.get(CONF_PATH_ARG)
        .expect("Missing argument - Configuration path file not fount"));
    monitor.setup().await;
    let monitor_state = Arc::new(monitor);

    let app = Router::new()
        .route("/api/v1/data/users", get(handlers::get_users))
        .route("/api/v1/data/files", get(handlers::get_remote_files))
        .route("/api/v1/data/files/:user_name", get(handlers::get_remote_file_by_user))
        .route("/api/v1/data/file", post(handlers::post_file))
        .route("/api/v1/user/login", post(handlers::login))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    format!("http://{}:3000", args.get(ORIGIN_ARG).expect("Missing argument - Origin"))
                    .parse::<HeaderValue>()
                    .unwrap()
                )
                .allow_methods([Method::GET, Method::POST]),
        )
        .layer(Extension(monitor_state));

    let listener = tokio::net::TcpListener::bind(ADDR_PORT).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
