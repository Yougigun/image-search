#![allow(clippy::redundant_pub_crate)]

use anyhow::Error;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use serde_json::json;
use xlib::{
    app::serve::serve_service,
    client::{PostgresClient, PostgresClientConfig},
};

use serde::{Deserialize, Serialize};
use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};
mod repo;

#[derive(Deserialize, Serialize, Clone)]
struct CreateFeedbackRequest {
    text: String,
    image_name: String,
    user_feedback: i32,
    model_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct CreateFeedbackResponse {
    id: i32,
}

#[derive(Clone)]
struct AppState {
    pub pg_client: Arc<PostgresClient>,
}

async fn create_feedback_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Response {
    // TODO: parse the jwt token to get the signed content of image search result
    let repo = repo::Repo::new(state.pg_client);
    let r = repo
        .create_feedback(
            payload.text,
            payload.image_name,
            payload.model_name,
            payload.user_feedback,
        )
        .await;
    match r {
        Ok(id) => axum::Json(id).into_response(),
        Err(_) => Json(json!("create feedback failed")).into_response(),
    }
}



async fn search_image_handler(State(state): State<AppState>) -> Response {
    
    
    return StatusCode::OK.into_response();
}
async fn init_db() -> PostgresClient {
    let db_config = PostgresClientConfig {
        hostname: env::var("DATABASE_HOSTNAME").expect("DATABASE_HOSTNAME not found."),
        port: None,
        user: Some(env::var("DATABASE_USER").expect("DATABASE_USER not found.")),
        password: Some(env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD not found.")),
        db_name: "web-server".to_string(),
    };
    PostgresClient::build(&db_config).await.unwrap()
}

async fn start_web_server() {
    let db_client = init_db().await;

    let state = AppState {
        pg_client: Arc::new(db_client.clone()),
    };
    let app = Router::new()
        .route(
            "/api/v1/healthcheck",
            get(|| async { StatusCode::OK.into_response() }),
        )
        .route("/api/v1/search-image", post(search_image_handler))
        .route("/api/v1/create-feedback", post(create_feedback_handler))
        .with_state(state);

    let public_service = serve_service(
        app,
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000),
        "public image search service",
    );

    tokio::select! {
        _ = public_service => {}
    };
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stdout)
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_web_server());
}
