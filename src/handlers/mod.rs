use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use axum::{
    http::StatusCode,
    Extension,
    extract::Json,
    //debug_handler,
};

use crate::monitor::Monitor;

//#[debug_handler]
pub async fn get_users(monitor_state: Extension<Arc<Monitor>>) -> String {
    let mut users = Vec::new();

    for machine in &monitor_state.config.remotes {
       users.push(&machine.usr);
    }

    serde_json::to_string(&users).unwrap()
}

//#[debug_handler]
pub async fn get_remote_files(monitor_state: Extension<Arc<Monitor>>) -> (StatusCode, String) {
    let mut files = HashMap::new();

    for machine in &monitor_state.config.remotes {
        if let Some(file_data) = machine.read_file_data().await {
            files.insert(&machine.usr, file_data);
        }
    }

    if let Ok(ret) = serde_json::to_string(&files) {
        return (StatusCode::OK, ret);
    }

    (StatusCode::INTERNAL_SERVER_ERROR, String::new())
}

#[derive(Deserialize)]
//#[debug_handler]
pub struct PostFile {
    user: String,
    file_ctx: String
}

pub async fn post_file(
    monitor_state: Extension<Arc<Monitor>>,
    Json(query): Json<PostFile>
) -> StatusCode {
    if let Some(machine) = &monitor_state.get_machine_by_name(&query.user) {
        if let Ok(_) = machine.write_file(&query.file_ctx).await {
            return StatusCode::OK;
        }
    }

    StatusCode::INTERNAL_SERVER_ERROR
}
