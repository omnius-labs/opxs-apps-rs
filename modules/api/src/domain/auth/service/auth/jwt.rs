use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::shared::AppError;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(sub: &str, expires_in: Duration) -> Self {
        let iat = Utc::now();
        let exp = iat + expires_in;

        Self {
            sub: sub.to_string(),
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn sign(secret: &str, sub: &str, expires_in: Duration) -> Result<String, AppError> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(sub, expires_in),
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify(secret: &str, token: &str) -> Result<Claims, AppError> {
    Ok(jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|n| n.claims)?)
}
