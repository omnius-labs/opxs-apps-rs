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
                    .route("/health", get(routes::health))
                    .with_state(state.clone())
                    .nest_service("/auth", routes::auth(state)),
            )
            .layer(cors);

        if cfg!(debug_assertions) {
            // Run app on local server
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

            tracing::debug!("listening on: http://{}/api/docs", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await?;
        } else {
            // Run app on AWS Lambda
            let app = tower::ServiceBuilder::new()
                .layer(axum_aws_lambda::LambdaLayer::default())
                .service(app);

            lambda_http::run(app).await.unwrap();
        }

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
            routes::auth::RegisterOutput,
            routes::auth::LoginInput,
            domain::auth::model::User,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "opxs", description = "Opxs API")
    )
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
