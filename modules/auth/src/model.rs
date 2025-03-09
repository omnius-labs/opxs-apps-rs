use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthToken {
    pub access_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub refresh_token: String,
    pub refresh_token_expires_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_authentication_type")]
pub enum UserAuthenticationType {
    Email,
    Provider,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role")]
pub enum UserRole {
    Admin,
    User,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Validate, ToSchema)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Validate, ToSchema)]
pub struct EmailUser {
    pub id: String,
    pub name: String,
    pub role: UserRole,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub salt: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
