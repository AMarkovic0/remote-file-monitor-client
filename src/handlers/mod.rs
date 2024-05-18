use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use axum::{
    Extension,
    http::StatusCode,
    extract::{Json, Path},
    //debug_handler,
};
use axum_extra::{
    headers::{
        Authorization,
        authorization::Bearer
    },
    TypedHeader,
};


use crate::monitor::Monitor;
use crate::remote_session::BoxResult;

const TEST_TOKEN: &'static str = "thisismydefaulttesttoken123!!";

fn verify_auth(token: &str) -> BoxResult<()> {
    if token == TEST_TOKEN {
        return Ok(())
    }

    Err(Box::from("Authorization failed"))
}

//#[debug_handler]
pub async fn get_users(monitor_state: Extension<Arc<Monitor>>) -> String {
    let mut users = Vec::new();

    for machine in &monitor_state.config.remotes {
       users.push(&machine.usr);
    }

    serde_json::to_string(&users).unwrap()
}

//#[debug_handler]
pub async fn get_remote_files(
    monitor_state: Extension<Arc<Monitor>>,
    auth: TypedHeader<Authorization<Bearer>>
) -> Result<String, StatusCode> {
    if let Err(_) = verify_auth(auth.token()) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let mut files = HashMap::new();

    for machine in &monitor_state.config.remotes {
        if let Some(file_data) = machine.read_file_data().await {
            files.insert(&machine.usr, file_data);
        }
    }

    if let Ok(ret) = serde_json::to_string(&files) {
        return Ok(ret);
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

//#[debug_handler]
pub async fn get_remote_file_by_user(
    monitor_state: Extension<Arc<Monitor>>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(user_name): Path<String>
) -> Result<String, StatusCode> {
    if let Err(_) = verify_auth(auth.token()) {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    if let Some(machine) = &monitor_state.get_machine_by_name(&user_name) {
        if let Some(file_data) = machine.read_file_data().await {
            return Ok(file_data);
        }
    }

    Err(StatusCode::NOT_FOUND)
}


#[derive(Deserialize)]
pub struct PostFile {
    user: String,
    file_ctx: String
}

//#[debug_handler]
pub async fn post_file(
    monitor_state: Extension<Arc<Monitor>>,
    auth: TypedHeader<Authorization<Bearer>>,
    Json(query): Json<PostFile>
) -> StatusCode {
    if let Err(_) = verify_auth(auth.token()) {
        return StatusCode::METHOD_NOT_ALLOWED;
    }

    if let Some(machine) = &monitor_state.get_machine_by_name(&query.user) {
        if let Ok(_) = machine.write_file(&query.file_ctx).await {
            return StatusCode::OK;
        }
    }

    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Deserialize)]
pub struct RequestUser {
    username: String,
    password: String
}

#[derive(Deserialize, Serialize)]
pub struct ResponseUser {
    username: String,
    id: u32,
    token: String
}

//#[debug_handler]
pub async fn login(
    Json(user): Json<RequestUser>
) -> Result<Json<ResponseUser>, StatusCode> {
    if user.username == "admin" && user.password == "default" {
        let resp = ResponseUser {
            username: user.username,
            id: 1,
            token: TEST_TOKEN.to_string()
        };

        return Ok(Json(resp));
    }

    Err(StatusCode::NOT_FOUND)
}
