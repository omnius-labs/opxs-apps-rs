use axum::{
    Json, RequestPartsExt as _, async_trait,
    extract::{FromRequest, FromRequestParts, Request, rejection::JsonRejection},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use omnius_opxs_auth::{crypto::jwt, model::User};
use omnius_opxs_base::AppError;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::shared::state::AppState;

#[async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts.extract::<TypedHeader<Authorization<Bearer>>>().await?;

        let access_token = bearer.token();
        let now = state.service.clock.now();
        let claims = jwt::verify(&state.conf.auth.jwt.secret.current, access_token, now)?;

        let user_id = claims.sub;
        let user = state.service.user.get_user(&user_id).await?;

        Ok(user)
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/validator/src/main.rs
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
