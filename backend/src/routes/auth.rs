use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{generate_token, hash_password, require_role, verify_password, AuthenticatedUser},
    errors::AppResult,
    storage::UserSummary,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub user_id: String,
    pub roles: Vec<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/admin/users", get(list_users).post(create_user))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<LoginResponse>)> {
    let Some(user) = state.storage.get_user_with_roles(&request.email).await? else {
        return Err("Invalid credentials".into());
    };

    let password_ok = verify_password(&user.password_hash, &request.password)?;
    if !password_ok {
        return Err("Invalid credentials".into());
    }

    let token = generate_token(&user, state.jwt_secret(), state.token_ttl_hours())?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            token,
            roles: user.roles,
        }),
    ))
}

async fn create_user(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<CreateUserResponse>)> {
    require_role(&user, &["admin"])?;

    let password_hash = hash_password(&request.password)?;
    let user_id = Uuid::new_v4().to_string();
    let mut assigned_roles = request.roles.clone();
    if assigned_roles.is_empty() {
        assigned_roles.push("viewer".to_string());
    }

    state
        .storage
        .create_user(&user_id, &request.email, &password_hash, &assigned_roles)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            user_id,
            roles: assigned_roles,
        }),
    ))
}

async fn list_users(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<UserSummary>>> {
    require_role(&user, &["admin"])?;
    let users = state.storage.list_users().await?;
    Ok(Json(users))
}
