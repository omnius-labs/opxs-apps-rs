use axum::{extract::State, response::Redirect, routing::get, Json, Router};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    domain,
    interface::routes::auth,
    shared::{AppError, AppState},
};

pub struct WebServer;

impl WebServer {
    pub async fn serve(state: AppState) -> anyhow::Result<()> {
        let cors = CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = Router::new()
            .route("/", get(|| async { Redirect::permanent("/api/docs") }))
            .merge(SwaggerUi::new("/api/docs").url("/api/api-doc/openapi.json", ApiDoc::openapi()))
            .nest_service(
                "/api",
                Router::new().route("/", get(|| async { Redirect::permanent("/api/docs") })).nest_service(
                    "/v1",
                    Router::new()
                        .route("/health", get(health))
                        .with_state(state.clone())
                        .nest_service("/auth", auth::gen_service(state.clone())),
                ),
            )
            .layer(cors);

        // Run app on local server
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

        tracing::info!("listening on: http://localhost:8080/api/docs");
        axum::Server::bind(&addr).serve(app.into_make_service()).await?;

        Ok(())
    }
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200)
    )
)]
#[allow(unused)]
pub async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let ret = state.service.health.check().await?;
    Ok(Json(ret))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        auth::me,
        auth::email::register,
        auth::email::login,
        auth::google::nonce,
        auth::google::register,
        auth::google::login,
    ),
    components(
        schemas(
            domain::auth::model::EmailUser,
            auth::email::RegisterInput,
            auth::email::LoginInput,
            auth::google::NonceOutput,
            auth::google::RegisterInput,
            auth::google::LoginInput,
        )
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                    utoipa::openapi::security::HttpAuthScheme::Bearer,
                )),
            )
        }
    }
}
