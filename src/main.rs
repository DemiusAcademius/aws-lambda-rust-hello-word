use http::Response;
use lambda_http::{http::StatusCode, run, service_fn, Error, IntoResponse, Request, RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body = event.payload::<MyPayload>()?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "Hello World",
              "payload": body,
              "payload-exists": body.is_some()
            })
            .to_string(),
        )
        .map_err(Box::new)?;

    Ok(response)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyPayload {
    pub prop1: String,
    pub prop2: String,
}
