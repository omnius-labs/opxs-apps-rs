use std::sync::LazyLock;

use axum::{
    Router,
    extract::Request,
    http::{HeaderName, HeaderValue},
    response::{Redirect, Response},
    routing::get,
};
use omnius_opxs_base::util::Terminable as _;
use parking_lot::Mutex;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Span;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{interface::features::*, prelude::*, shared::state::AppState};

pub struct WebServer;

impl WebServer {
    pub async fn serve(state: AppState) -> Result<()> {
        let cors_layer = CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let x_request_id = HeaderName::from_static("x-request-id");
        let set_request_id_layer = SetRequestIdLayer::new(x_request_id.clone(), MyRequestId::new());
        let propagate_request_id_layer = PropagateRequestIdLayer::new(x_request_id);

        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let x_request_id = HeaderName::from_static("x-request-id");
                let request_id = request.headers().get(&x_request_id).and_then(|id| id.to_str().ok()).unwrap_or("unknown");

                tracing::info_span!(
                    "http_request",
                    request_id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            })
            .on_response(|response: &Response<_>, latency: std::time::Duration, _span: &Span| {
                tracing::info!(
                    status = %response.status().as_u16(),
                    latency = %format!("{:.3}ms", latency.as_secs_f64() * 1000.0),
                    "http_response"
                );
            });

        let app = Router::new()
            .route("/", get(|| async { Redirect::permanent("/api/docs") }))
            .merge(SwaggerUi::new("/api/docs").url("/api/api-doc/openapi.json", ApiDoc::openapi()))
            .nest_service(
                "/api",
                Router::new().route("/", get(|| async { Redirect::permanent("/api/docs") })).nest_service(
                    "/v1",
                    Router::new()
                        .route("/health", get(health::check))
                        .with_state(state.clone())
                        .nest_service("/auth", auth::gen_service(state.clone()))
                        .nest_service("/file-convert", file_convert::gen_service(state.clone())),
                ),
            )
            .layer(cors_layer)
            .layer(trace_layer)
            .layer(propagate_request_id_layer)
            .layer(set_request_id_layer);

        if cfg!(debug_assertions) {
            // Run app on local server
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        } else {
            // Run app on AWS Lambda
            let app = tower::ServiceBuilder::new().layer(axum_aws_lambda::LambdaLayer::default()).service(app);
            lambda_http::run(app)
                .await
                .map_err(|e| Error::new(ErrorKind::UnexpectedError).source(e))?;
        }

        state.service.terminate().await;

        Ok(())
    }
}

static TSID_PROVIDER: LazyLock<Mutex<Box<dyn omnius_core_base::tsid::TsidProvider + Send + Sync>>> = LazyLock::new(|| {
    Mutex::new(Box::new(omnius_core_base::tsid::TsidProviderImpl::new(
        omnius_core_base::clock::ClockUtc,
        omnius_core_base::random_bytes::RandomBytesProviderImpl::new(),
        16,
    )))
});

#[derive(Clone)]
pub struct MyRequestId {}

impl MyRequestId {
    pub fn new() -> Self {
        MyRequestId {}
    }
}

impl MakeRequestId for MyRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = TSID_PROVIDER.lock().as_mut().create().to_string();
        let id = HeaderValue::from_str(&id).unwrap();
        Some(RequestId::new(id))
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
            omnius_opxs_file_convert::FileConvertJobStatus,
            omnius_opxs_file_convert::FileConvertImageInputFileType,
            omnius_opxs_file_convert::FileConvertImageOutputFileType,
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
