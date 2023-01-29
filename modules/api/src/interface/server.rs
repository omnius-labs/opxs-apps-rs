use axum::{Router, Server};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{interface::routes, shared::AppState};

pub struct WebServer;

impl WebServer {
    pub async fn serve(state: &AppState) -> anyhow::Result<()> {
        let cors = CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = Router::new()
            .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
            .nest_service("/auth", routes::auth(state.clone()))
            .layer(cors);

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

        tracing::debug!("listening on http://{}", addr);

        Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");

        Ok(())
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::auth::register,
    ),
    components(
        schemas(
            routes::auth::RegisterParam,
            routes::auth::Registered
        )
    ),
    tags(
        (name = "pxtv", description = "Pxtv API")
    )
)]
struct ApiDoc;
