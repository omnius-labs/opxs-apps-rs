use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow)]
pub struct User {
    pub name: String,
    pub email: String,
    pub password: String,
    pub salt: String,
}
