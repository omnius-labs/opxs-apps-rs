use std::sync::Arc;

use axum::{response::Redirect, routing::get, Router};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{domain, interface::routes, shared::AppState};

pub struct WebServer;

impl WebServer {
    pub async fn serve(state: &Arc<AppState>) -> anyhow::Result<()> {
        let cors = CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = Router::new()
            .route("/", get(|| async { Redirect::permanent("/api/docs") }))
            .merge(SwaggerUi::new("/api/docs").url("/api/api-doc/openapi.json", ApiDoc::openapi()))
            .nest_service(
                "/api",
                Router::new()
                    .route("/", get(|| async { Redirect::permanent("/api/docs") }))
                    .nest_service(
                        "/v1",
                        Router::new()
                            .route("/health", get(routes::health))
                            .with_state(state.clone())
                            .nest_service("/auth", routes::auth(state)),
                    ),
            )
            .layer(cors);

        // Run app on local server
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

        tracing::info!("listening on: http://localhost:8080/api/docs");
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health,
        routes::auth::register,
        routes::auth::login,
        routes::auth::me,
    ),
    components(
        schemas(
            routes::auth::RegisterInput,
            routes::auth::LoginOutput,
            routes::auth::LoginInput,
            domain::auth::model::User,
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
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}
