use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub password: String,
}
