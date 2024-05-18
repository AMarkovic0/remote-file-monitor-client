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
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;

use crate::monitor::Monitor;

const HOST: &'static str = "HOST";
const PORT: &'static str = "PORT";
const CONF_PATH: &'static str = "CONF_PATH";
const ORIGIN: &'static str = "ORIGIN";
const DATABASE_URL: &'static str = "DATABASE_URL";

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let conf_path = env::var(CONF_PATH).expect("CONF_PATH is not set in .env file");
    let origin = env::var(ORIGIN).expect("ORIGIN is not set in .env file");
    let host = env::var(HOST).expect("HOST is not set in .env file");
    let port = env::var(PORT).expect("PORT is not set in .env file");
    let db_url = env::var(DATABASE_URL).expect("DATABASE_URL is not set in .env file");
    let server_url = format!("{host}:{port}");

    let mut monitor = Monitor::new(&conf_path);
    monitor.setup().await;
    let monitor_state = Arc::new(monitor);

    let pool = SqlitePool::connect(&db_url).await.expect("Failed connecting to database");

    let app = Router::new()
        .route("/api/v1/data/users", get(handlers::get_users))
        .route("/api/v1/data/files", get(handlers::get_remote_files))
        .route("/api/v1/data/files/:user_name", get(handlers::get_remote_file_by_user))
        .route("/api/v1/data/file", post(handlers::post_file))
        .route("/api/v1/user/login", post(handlers::login))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    format!("http://{origin}:{port}")
                    .parse::<HeaderValue>()
                    .unwrap()
                )
                .allow_methods([Method::GET, Method::POST]),
        )
        .layer(Extension(monitor_state));

    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
