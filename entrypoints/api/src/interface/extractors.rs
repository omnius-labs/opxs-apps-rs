use axum::{
    Json, RequestPartsExt as _,
    extract::{FromRequest, FromRequestParts, Request, rejection::JsonRejection},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::de::DeserializeOwned;
use validator::Validate;

use omnius_opxs_auth::{crypto::jwt, model::User};

use crate::{prelude::*, shared::state::AppState};

impl FromRequestParts<AppState> for User {
    type Rejection = ApiErrorCode;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> std::result::Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = ?e);
                return Err(ApiErrorCode::InternalServerError);
            }
        };

        let access_token = bearer.token();
        let now = state.service.clock.now();
        let claims = match jwt::verify(&state.conf.auth.jwt.secret.current, access_token, now) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = ?e);
                return Err(ApiErrorCode::Unauthorized);
            }
        };

        let user_id = claims.sub;
        let user = match state.service.user.get_user(&user_id).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = ?e);
                return Err(ApiErrorCode::Unauthorized);
            }
        };

        Ok(user)
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/validator/src/main.rs
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiErrorCode;

    async fn from_request(req: Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let Json(value) = match Json::<T>::from_request(req, state).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = ?e);
                return Err(ApiErrorCode::InternalServerError);
            }
        };
        if let Err(e) = value.validate() {
            warn!(error = ?e);
            return Err(ApiErrorCode::InvalidRequest);
        }

        Ok(ValidatedJson(value))
    }
}
