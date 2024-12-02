use axum::{response::Redirect, routing::get, Router};
use omnius_core_base::terminable::Terminable;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    interface::{
        health,
        routes::{auth, file_convert},
    },
    shared::state::AppState,
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
                Router::new()
                    .route("/", get(|| async { Redirect::permanent("/api/docs") }))
                    .nest_service(
                        "/v1",
                        Router::new()
                            .route("/health", get(health::check))
                            .with_state(state.clone())
                            .nest_service("/auth", auth::gen_service(state.clone()))
                            .nest_service(
                                "/file-convert",
                                file_convert::gen_service(state.clone()),
                            ),
                    ),
            )
            .layer(cors);

        if cfg!(debug_assertions) {
            // Run app on local server
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        } else {
            // Run app on AWS Lambda
            let app = tower::ServiceBuilder::new()
                .layer(axum_aws_lambda::LambdaLayer::default())
                .service(app);
            lambda_http::run(app)
                .await
                .map_err(|_| anyhow::anyhow!("lambda_http run error"))?;
        }

        state.service.terminate().await?;

        Ok(())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health::check,
        auth::me,
        auth::email::register,
        auth::email::login,
        auth::google::nonce,
        auth::google::register,
        auth::google::login,
        file_convert::image::upload,
        file_convert::image::status,
    ),
    components(
        schemas(
            auth::email::RegisterInput,
            auth::email::LoginInput,
            auth::google::NonceOutput,
            auth::google::RegisterInput,
            auth::google::LoginInput,
            file_convert::image::UploadInput,
            file_convert::image::UploadOutput,
            file_convert::image::StatusInput,
            file_convert::image::StatusOutput,
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
