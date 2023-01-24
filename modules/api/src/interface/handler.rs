use std::sync::Arc;

use axum::http::StatusCode;
use axum::Extension;

use super::docs::docs_routes;
use super::errors::AppError;
use super::extractors::Json;
use super::state::AppState;

pub struct Handler {}

impl Handler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        aide::gen::on_error(|error| {
            println!("{error}");
        });

        aide::gen::extract_schemas(true);

        let state = AppState::default();

        let mut api = aide::openapi::OpenApi::default();
        let cors = tower_http::cors::CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = aide::axum::ApiRouter::new()
            .nest_api_service("/docs", docs_routes(state.clone()))
            .finish_api_with(&mut api, Handler::api_docs)
            .layer(Extension(Arc::new(api)))
            .layer(cors)
            .with_state(state);

        println!("http://127.0.0.1:3000/docs");

        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");

        Ok(())
    }

    fn api_docs(api: aide::transform::TransformOpenApi) -> aide::transform::TransformOpenApi {
        api.title("pxtv-api")
            .tag(aide::openapi::Tag {
                name: "todo".into(),
                description: Some("Todo Management".into()),
                ..Default::default()
            })
            .default_response_with::<Json<AppError>, _>(|res| {
                res.example(AppError {
                    error: "some error happened".to_string(),
                    error_details: None,
                    status: StatusCode::IM_A_TEAPOT,
                })
            })
    }
}
