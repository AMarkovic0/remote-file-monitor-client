use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;
use axum::{
    Extension,
    extract::Json
};

use crate::monitor::Monitor;

pub async fn get_users(monitor_state: Extension<Arc<Monitor>>) -> String {
    let mut users = Vec::new();

    for machine in &monitor_state.config.remotes {
       users.push(&machine.usr);
    }

    serde_json::to_string(&users).unwrap()
}

pub async fn get_remotes(monitor_state: Extension<Arc<Monitor>>) -> String {
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
pub struct PostFile {
    user: String,
    file_ctx: String
}

pub async fn post_file(monitor_state: Extension<Arc<Monitor>>, Json(query): Json<PostFile>) {
    if let Some(machine) = &monitor_state.get_machine_by_name(&query.user) {
        machine.write_file(&query.file_ctx).await;
    }
}
