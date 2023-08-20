use chrono::{DateTime, Duration, Utc};
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
    pub fn new(sub: &str, iat: DateTime<Utc>, exp: DateTime<Utc>) -> Self {
        Self {
            sub: sub.to_string(),
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn sign(secret: &str, sub: &str, expires_in: Duration, iat: DateTime<Utc>) -> Result<String, AppError> {
    let exp = iat + expires_in;
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(sub, iat, exp),
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify(secret: &str, token: &str) -> Result<Claims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();
    Ok(jsonwebtoken::decode(token, &key, &validation).map(|token| token.claims)?)
}
