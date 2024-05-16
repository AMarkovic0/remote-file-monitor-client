mod remote_session;
mod remote_machine;
mod monitor;

use std::env;
use std::sync::Arc;
use std::collections::HashMap;

use axum::{
    routing::get,
    routing::post,
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
        .route("/users", get(get_users))
        .route("/files", get(get_remotes))
        //.route("/file", post(post_file))
        .layer(Extension(monitor_state));

    let listener = tokio::net::TcpListener::bind(ADDR_PORT).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

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

//async fn post_file(user: &str, file_ctx: &str, monitor_state: Extension<Arc<Monitor>>) {
//    if let Some(machine) = &monitor_state.get_machine_by_name(user) {
//        machine.write_file(file_ctx).await;
//    }
//}
