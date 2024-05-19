use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::env;

use serde::{Deserialize, Serialize};
use axum::{
    Extension,
    http::StatusCode,
    extract::{Json, Path, State},
    //debug_handler,
};
use axum_extra::{
    headers::{
        Authorization,
        authorization::Bearer
    },
    TypedHeader,
};
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::{
   SqlitePool,
   Row,
};

use crate::monitor::Monitor;
use crate::remote_session::BoxResult;

const SECRET_SIGNING_KEY: &'static str = "thisismydefaulttestkey123!!";
const AUTH_REQUIRED_VAR: &'static str = "AUTH_REQUIRED";

#[derive(Serialize, Deserialize)]
pub struct JwtPayload {
    pub sub: String,
    pub exp: usize,
}

impl JwtPayload {
    pub fn new(sub: String) -> Self {
        // expires by default in 60 minutes from now
        let exp = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60))
            .expect("valid timestamp")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("valid duration")
            .as_secs() as usize;

        JwtPayload { sub, exp }
    }

    pub fn verify(token: &str) -> BoxResult<String> {
        if env::var(AUTH_REQUIRED_VAR).expect("AUTH_REQUIRED is not set in .env file") == "true" {
            return Ok("no_auth".to_string());
        }

        let decoding_key = DecodingKey::from_secret(SECRET_SIGNING_KEY.as_bytes());

        let Ok(jwt) =
            jsonwebtoken::decode::<JwtPayload>(token, &decoding_key, &Validation::default())
        else {
            return Err("Unauthorized access".into());
        };

        Ok(jwt.claims.sub)
    }
}

//#[debug_handler]
pub async fn get_users(
    monitor_state: Extension<Arc<Monitor>>,
    auth: TypedHeader<Authorization<Bearer>>
) -> Result<String, StatusCode> {
    if let Err(_) = JwtPayload::verify(auth.token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut users = Vec::new();

    for machine in &monitor_state.config.remotes {
       users.push(&machine.usr);
    }

    Ok(serde_json::to_string(&users).unwrap())
}

//#[debug_handler]
pub async fn get_remote_files(
    monitor_state: Extension<Arc<Monitor>>,
    auth: TypedHeader<Authorization<Bearer>>
) -> Result<String, StatusCode> {
    if let Err(_) = JwtPayload::verify(auth.token()) {
        return Err(StatusCode::UNAUTHORIZED);
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
    if let Err(_) = JwtPayload::verify(auth.token()) {
        return Err(StatusCode::UNAUTHORIZED);
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
    if let Err(_) = JwtPayload::verify(auth.token()) {
        return StatusCode::UNAUTHORIZED;
    }

    if let Some(machine) = &monitor_state.get_machine_by_name(&query.user) {
        if let Ok(_) = machine.write_file(&query.file_ctx).await {
            return StatusCode::OK;
        }
    }

    StatusCode::INTERNAL_SERVER_ERROR
}

#[derive(Deserialize, Debug)]
pub struct RequestUser {
    username: String,
    password: String
}

#[derive(Deserialize, Serialize)]
pub struct ResponseUser {
    username: String,
    token: String
}

//#[debug_handler]
pub async fn login(
    State(state_db): State<SqlitePool>,
    Json(user): Json<RequestUser>
) -> Result<Json<ResponseUser>, StatusCode> {
    let recs = sqlx::query(
        "
SELECT username, password
FROM clients WHERE username = ?
        "
    )
    .bind(&user.username)
    .fetch_all(&state_db)
    .await.expect("Failed to fetch user from database");

    if let Some(rec) = recs.get(0) {
        let username: &str = rec.try_get("username").expect("Failed parsing sql row");
        let password: &str = rec.try_get("password").expect("Failed parsing sql row");

        if user.username != username || user.password != password {
            return Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        return Err(StatusCode::NOT_FOUND);
    }

    let Ok(jwt) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &JwtPayload::new(user.username.clone()),
        &jsonwebtoken::EncodingKey::from_secret(SECRET_SIGNING_KEY.as_bytes()),
    ) else {
        return  Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(Json(ResponseUser {
        username: user.username,
        token: jwt
    }))
}
