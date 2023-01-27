use std::sync::Arc;

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{http::StatusCode, Extension, Server};
use tower_http::cors::CorsLayer;

use super::{docs::docs_routes, errors::AppError, extractors::Json};

use crate::domain::state::AppState;

pub struct WebServer {
    pub state: AppState,
}

impl WebServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        aide::gen::on_error(|error| {
            println!("{error}");
        });

        aide::gen::extract_schemas(true);

        let mut api = OpenApi::default();
        let cors = CorsLayer::new().allow_origin(tower_http::cors::Any);

        let app = ApiRouter::new()
            .nest_api_service("/docs", docs_routes(self.state.clone()))
            .finish_api_with(&mut api, WebServer::api_docs)
            .layer(Extension(Arc::new(api)))
            .layer(cors)
            .with_state(self.state.clone());

        println!("http://127.0.0.1:3000/docs");

        Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");

        Ok(())
    }

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
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
