use std::sync::Arc;

use aide::axum::routing::{get, get_with};
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::OpenApi;
use aide::redoc::Redoc;
use axum::response::IntoResponse;
use axum::Extension;

use crate::domain::AppState;

use super::extractors::Json;

pub fn docs_routes(state: AppState) -> ApiRouter {
    aide::gen::infer_responses(true);

    let router = ApiRouter::new()
        .api_route(
            "/",
            get_with(
                Redoc::new("/docs/private/api.json")
                    .with_title("Aide Axum")
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
        )
        .route("/private/api.json", get(serve_docs))
        .with_state(state);

    aide::gen::infer_responses(false);

    router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}
