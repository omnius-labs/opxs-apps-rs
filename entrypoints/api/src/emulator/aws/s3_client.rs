use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, put},
};
use chrono::{DateTime, Duration, Utc};
use futures::{FutureExt, TryStreamExt};
use tokio::{
    fs,
    io::BufWriter,
    sync::{Mutex as TokioMutex, mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::io::{ReaderStream, StreamReader};
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use url::Url;

use omnius_core_cloud::{Result, aws::s3::S3Client};
use omnius_opxs_base::util::Terminable;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct S3ClientEmulatorOption {
    pub base_url: Url,
    pub listen_addr: SocketAddr,
    pub working_dir: PathBuf,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct S3ClientEmulatorState {
    pub working_dir: PathBuf,
    pub put_event_sender: mpsc::Sender<String>,
}

#[allow(unused)]
pub struct S3ClientEmulator {
    pub put_event_receiver: Arc<TokioMutex<mpsc::Receiver<String>>>,
    terminate_signal_sender: Box<TokioMutex<Option<oneshot::Sender<()>>>>,
    join_handle: Box<TokioMutex<Option<JoinHandle<()>>>>,
    option: S3ClientEmulatorOption,
}

#[allow(unused)]
impl S3ClientEmulator {
    pub fn new(option: S3ClientEmulatorOption) -> Result<Self> {
        let (terminate_signal_sender, terminate_signal_receiver) = oneshot::channel::<()>();
        let (put_event_sender, put_event_receiver) = mpsc::channel::<String>(32);

        let state = S3ClientEmulatorState {
            put_event_sender,
            working_dir: option.working_dir.clone(),
        };
        let join_handle = tokio::spawn(async move {
            let cors = CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any);
            let app = Router::new()
                .route("/", put(put_content))
                .route("/", get(get_content))
                .with_state(state)
                .layer(cors);
            let listener = tokio::net::TcpListener::bind(option.listen_addr).await.unwrap();
            let serve = axum::serve(listener, app).with_graceful_shutdown(async {
                terminate_signal_receiver.await.ok();
            });
            serve.await.unwrap();
        });

        Ok(Self {
            put_event_receiver: Arc::new(TokioMutex::new(put_event_receiver)),
            terminate_signal_sender: Box::new(TokioMutex::new(Some(terminate_signal_sender))),
            join_handle: Box::new(TokioMutex::new(Some(join_handle))),
            option,
        })
    }
}

#[async_trait]
impl Terminable for S3ClientEmulator {
    async fn terminate(&self) {
        if let Some(sender) = self.terminate_signal_sender.lock().await.take() {
            let _ = sender.send(());
        }

        if let Some(j) = self.join_handle.lock().await.take() {
            if let Err(e) = j.fuse().await {
                error!("{:?}", e);
            }
        }
    }
}

async fn get_content(Query(params): Query<GetContentQuery>, State(state): State<S3ClientEmulatorState>) -> impl IntoResponse {
    let file_path = state.working_dir;
    let file_path = file_path.join(params.key.replace("/", "_"));
    let body = match tokio::fs::File::open(file_path).await {
        Ok(file) => Body::from_stream(ReaderStream::new(file)),
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let encoded_file_name = urlencoding::encode(&params.file_name).to_string();
    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"; filename*=UTF-8''{}", &params.file_name, encoded_file_name),
        ),
    ];
    Ok((headers, body))
}

#[derive(serde::Deserialize)]
struct GetContentQuery {
    key: String,
    file_name: String,
}

async fn put_content(
    Query(params): Query<PutContentQuery>,
    State(state): State<S3ClientEmulatorState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let stream = request
        .into_body()
        .into_data_stream()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let mut body_reader = StreamReader::new(stream);

    info!("put_content: key={}", params.key);

    let file_path = state.working_dir;
    let file_path = file_path.join(params.key.replace("/", "_"));
    let mut file_writer = match tokio::fs::File::create(file_path).await {
        Ok(file) => BufWriter::new(file),
        _ => return Err(StatusCode::NOT_FOUND),
    };

    info!("put_content: key={}", params.key);

    tokio::io::copy(&mut body_reader, &mut file_writer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .put_event_sender
        .send(params.key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct PutContentQuery {
    key: String,
}

#[async_trait]
impl S3Client for S3ClientEmulator {
    async fn gen_get_presigned_uri(&self, key: &str, _start_time: DateTime<Utc>, _expires_in: Duration, file_name: &str) -> Result<String> {
        let encoded_key = urlencoding::encode(key).to_string();
        let encoded_file_name = urlencoding::encode(file_name).to_string();
        let mut url = self.option.base_url.clone();
        url.set_query(Some(&format!("key={}&file_name={}", encoded_key, encoded_file_name)));
        Ok(url.to_string())
    }

    async fn gen_put_presigned_uri(&self, key: &str, _start_time: DateTime<Utc>, _expires_in: Duration) -> Result<String> {
        let encoded_key = urlencoding::encode(key).to_string();
        let mut url = self.option.base_url.clone();
        url.set_query(Some(&format!("key={}", encoded_key)));
        Ok(url.to_string())
    }

    async fn get_object(&self, key: &str, destination: &Path) -> Result<()> {
        let file_path = PathBuf::from(&self.option.working_dir);
        let file_path = file_path.join(key.replace("/", "_"));
        let _ = fs::copy(file_path, destination).await?;
        Ok(())
    }

    async fn put_object(&self, key: &str, source: &Path) -> Result<()> {
        let file_path = PathBuf::from(&self.option.working_dir);
        let file_path = file_path.join(key.replace("/", "_"));
        let _ = fs::copy(source, file_path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use serial_test::serial;
    use tempfile::tempdir;
    use testresult::TestResult;
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[ignore]
    #[tokio::test]
    #[serial(s3_client_emulator)]
    async fn gen_get_presigned_uri_test() -> TestResult {
        let option = S3ClientEmulatorOption {
            base_url: "http://localhost:50000".parse()?,
            listen_addr: "0.0.0.0:50000".parse()?,
            working_dir: "/tmp".into(),
        };
        let s3_client = S3ClientEmulator::new(option)?;

        {
            let mut file = fs::File::create("/tmp/get_test").await?;
            file.write_all(b"get hello world.\n").await?;
            file.flush().await?;
        }

        let url = s3_client
            .gen_get_presigned_uri(
                "get_test",
                DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z").unwrap().into(),
                TimeDelta::zero(),
                "test_name",
            )
            .await?;

        let http_client = reqwest::Client::new();
        let content = http_client.get(url).send().await?.text().await?;

        println!("{}", content);

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    #[serial(s3_client_emulator)]
    async fn gen_put_presigned_uri_test() -> TestResult {
        let option = S3ClientEmulatorOption {
            base_url: "http://localhost:50000".parse()?,
            listen_addr: "0.0.0.0:50000".parse()?,
            working_dir: "/tmp".into(),
        };
        let s3_client = S3ClientEmulator::new(option)?;

        let url = s3_client
            .gen_put_presigned_uri(
                "put_test",
                DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z").unwrap().into(),
                TimeDelta::zero(),
            )
            .await?;

        let http_client = reqwest::Client::new();
        let _ = http_client.put(url).body("put hello world").send().await?;
        let content = fs::read_to_string("/tmp/put_test").await?;

        println!("{}", content);

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    #[serial(s3_client_emulator)]
    async fn get_content_test() -> TestResult {
        let working_dir = tempdir()?;

        let option = S3ClientEmulatorOption {
            base_url: "http://localhost:50000".parse()?,
            listen_addr: "0.0.0.0:50000".parse()?,
            working_dir: working_dir.path().to_path_buf(),
        };
        let s3_client = S3ClientEmulator::new(option)?;

        {
            let mut file = fs::File::create(working_dir.path().join("key")).await?;
            file.write_all(b"hello world.\n").await?;
            file.flush().await?;
        }

        let out_dir = tempdir()?;
        let out_file_path = out_dir.path().join("out");
        s3_client.get_object("key", &out_file_path).await?;

        let content = fs::read_to_string(&out_file_path).await?;

        println!("{}", content);

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    #[serial(s3_client_emulator)]
    async fn put_content_test() -> TestResult {
        let working_dir = tempdir()?;

        let option = S3ClientEmulatorOption {
            base_url: "http://localhost:50000".parse()?,
            listen_addr: "0.0.0.0:50000".parse()?,
            working_dir: working_dir.path().to_path_buf(),
        };
        let s3_client = S3ClientEmulator::new(option)?;

        let in_dir = tempdir()?;
        let in_file_path = in_dir.path().join("in");
        {
            let mut file = fs::File::create(&in_file_path).await?;
            file.write_all(b"hello world.\n").await?;
            file.flush().await?;
        }

        s3_client.put_object("key", &in_file_path).await?;

        let content = fs::read_to_string(working_dir.path().join("key")).await?;

        println!("{}", content);

        Ok(())
    }
}
