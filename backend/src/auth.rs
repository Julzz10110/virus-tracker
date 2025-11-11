use crate::{errors::AppError, state::AppState, storage::UserRecord};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub exp: i64,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| AppError::from(err.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(expected_hash: &str, password: &str) -> Result<bool, AppError> {
    let parsed_hash =
        PasswordHash::new(expected_hash).map_err(|err| AppError::from(err.to_string()))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_token(user: &UserRecord, secret: &str, ttl_hours: i64) -> Result<String, AppError> {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(ttl_hours.max(1));
    let claims = Claims {
        sub: user.id.clone(),
        roles: user.roles.clone(),
        exp: expiration.unix_timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::from(err.to_string()))?;

    Ok(token)
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|err| AppError::from(err.to_string()))?;

    Ok(data.claims)
}

#[derive(Clone, Debug)]
pub enum AuthKind {
    Public,
    ApiKey,
    Jwt,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub kind: AuthKind,
}

impl AuthenticatedUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|current| current == role)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }
}

pub fn require_role(user: &AuthenticatedUser, roles: &[&str]) -> Result<(), AppError> {
    if roles.is_empty() || user.has_any_role(roles) {
        Ok(())
    } else {
        Err(AppError::from("Forbidden"))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(expected) = state.api_key() {
            if let Some(provided) = parts
                .headers
                .get("x-api-key")
                .and_then(|value| value.to_str().ok())
            {
                if provided == expected {
                    return Ok(AuthenticatedUser {
                        user_id: None,
                        roles: vec!["admin".to_string()],
                        kind: AuthKind::ApiKey,
                    });
                }
            }
        }

        if let Some(header) = parts.headers.get("authorization") {
            if let Ok(value) = header.to_str() {
                if let Some(token) = value.strip_prefix("Bearer ") {
                    match decode_token(token, state.jwt_secret()) {
                        Ok(claims) => {
                            // Token expiration is handled by jsonwebtoken Validation
                            return Ok(AuthenticatedUser {
                                user_id: Some(claims.sub),
                                roles: claims.roles,
                                kind: AuthKind::Jwt,
                            });
                        }
                        Err(err) => {
                            return Err((
                                StatusCode::UNAUTHORIZED,
                                format!("Invalid token: {err:?}"),
                            )
                                .into_response())
                        }
                    }
                }
            }
        }

        if state.api_key().is_none() {
            Ok(AuthenticatedUser {
                user_id: None,
                roles: Vec::new(),
                kind: AuthKind::Public,
            })
        } else {
            Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response())
        }
    }
}
