use axum::{response::Redirect, routing::get, Router};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{domain, interface::routes::v1, shared::AppState};

pub struct WebServer;

impl WebServer {
    pub async fn serve(state: AppState) -> anyhow::Result<()> {
        let cors = CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = Router::new()
            .route("/", get(|| async { Redirect::permanent("/api/docs") }))
            .merge(SwaggerUi::new("/api/docs").url("/api/api-doc/openapi.json", ApiDoc::openapi()))
            .nest_service(
                "/api",
                Router::new()
                    .route("/", get(|| async { Redirect::permanent("/api/docs") }))
                    .nest_service("/v1", v1::gen_service(state)),
            )
            .layer(cors);

        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));

        tracing::info!("listening on: http://localhost:8080/api/docs");
        axum::Server::bind(&addr).serve(app.into_make_service()).await?;

        Ok(())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        v1::health,
        v1::auth::me,
        v1::auth::email::register,
        v1::auth::email::login,
        v1::auth::google::nonce,
        v1::auth::google::register,
        v1::auth::google::login,
    ),
    components(
        schemas(
            domain::auth::model::EmailUser,
            v1::auth::email::RegisterInput,
            v1::auth::email::LoginInput,
            v1::auth::google::NonceOutput,
            v1::auth::google::RegisterInput,
            v1::auth::google::LoginInput,
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
