use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

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

pub fn sign(secret: &str, sub: &str, exp: DateTime<Utc>, iat: DateTime<Utc>) -> Result<String> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(sub, iat, exp),
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify(secret: &str, token: &str, now: DateTime<Utc>) -> Result<Claims> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();
    let claims: Claims = jsonwebtoken::decode(token, &key, &validation).map(|token| token.claims)?;

    let expired_at = DateTime::from_timestamp(claims.exp, 0).ok_or_else(|| Error::builder().kind(ErrorKind::TokenExpired).build())?;
    if expired_at < now {
        return Err(Error::builder().kind(ErrorKind::TokenExpired).build());
    }

    Ok(claims)
}
