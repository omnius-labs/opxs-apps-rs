use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, TypedHeader},
    http::Request,
    Json,
};
use headers::{authorization::Bearer, Authorization};
use serde::de::DeserializeOwned;
use validator::Validate;

use omnius_opxs_auth::shared::{jwt, model::User};
use omnius_opxs_base::AppError;

use crate::shared::state::AppState;

#[async_trait]
impl<B> FromRequest<AppState, B> for User
where
    B: Send + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request<B>, state: &AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request(req, state).await?;

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
impl<T, S, B> FromRequest<S, B> for ValidatedJson<T>
where
    S: Send + Sync,
    B: Send + 'static,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;

        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
