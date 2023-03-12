use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow)]
pub struct User {
    pub name: String,
    pub email: String,
    pub encrypted_password_hash: String,
    pub encrypted_password_salt: String,
}
