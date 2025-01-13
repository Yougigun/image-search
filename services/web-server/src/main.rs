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
use tracing::info;
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

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use qdrant_client::qdrant::QueryPointsBuilder;
use qdrant_client::Qdrant;

#[derive(Deserialize, Serialize, Clone)]
struct CreateFeedbackRequest {
    jwt: String,
    user_feedback: i32,
}

#[derive(Deserialize, Serialize, Clone)]
struct CreateFeedbackResponse {
    id: i32,
}

#[derive(Clone)]
struct AppState {
    pub pg_client: Arc<PostgresClient>,
}

#[derive(Deserialize)]
struct SearchImageRequest {
    text: String,
}

#[derive(Serialize)]
struct SearchImageResponse {
    text: String,
    model_name: String,
    matches: Vec<ImageMatch>,
    jwt: String,
}

#[derive(Serialize)]
struct ImageMatch {
    image_name: String,
    score: f32,
}

// TODO: Inject this secret via environment variables and keep it secure for production deployment
const JWT_SECRET: &str = "jwt_secret";
async fn create_feedback_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Response {
    let claims = match decode::<Claims>(
        &payload.jwt,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("Error decoding JWT: {}", e);
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    let repo = repo::Repo::new(state.pg_client);
    let r = repo
        .create_feedback(
            claims.claims.image_name,
            claims.claims.text,
            claims.claims.model_name,
            payload.user_feedback,
        )
        .await;
    match r {
        Ok(id) => axum::Json(id).into_response(),
        Err(_) => Json(json!("create feedback failed")).into_response(),
    }
}
#[derive(Serialize, Deserialize)]
struct Claims {
    exp: i64,
    iat: i64,
    image_name: String,
    text: String,
    model_name: String,
    score: f32,
}

fn create_jwt(
    secret: &str,
    image_name: String,
    text: String,
    model_name: String,
    score: f32,
) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp();

    let claims = Claims {
        exp: expiration,
        iat: chrono::Utc::now().timestamp(),
        image_name,
        text,
        model_name,
        score,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

async fn search_image_handler(
    State(state): State<AppState>,
    Json(payload): Json<SearchImageRequest>,
) -> Response {
    let client = reqwest::Client::new();
    let qdrant_client = Qdrant::from_url("http://qdrant:6334").build().unwrap();
    let collection_name = "clip_images_collection";
    let clip_request = serde_json::json!({
        "text": payload.text
    });

    let response = match client
        .post("http://clip-model:8000/api/v1/clip/text-to-vector")
        .json(&clip_request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let text_vector_response = match response.json::<serde_json::Value>().await {
        Ok(json) => json,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let text_vector: Vec<f32> = match text_vector_response.get("vector") {
        Some(v) => v
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_f64().map(|x| x as f32))
            .collect(),
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let query = QueryPointsBuilder::new(collection_name)
        .query(text_vector)
        .with_payload(true);

    let search_result = qdrant_client.query(query).await.unwrap();
    let first_result = match search_result.result.first() {
        Some(r) => r,
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let image_name = match first_result.payload.get("image_name") {
        Some(v) => match v.as_str() {
            Some(s) => s.to_string(),
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let score = first_result.score;
    let mut response = SearchImageResponse {
        text: payload.text.clone(),
        model_name: "CLIP".to_string(),
        matches: vec![ImageMatch {
            image_name: image_name.clone(),
            score,
        }],
        jwt: String::new(),
    };

    let jwt = create_jwt(
        JWT_SECRET,
        image_name,
        payload.text,
        "CLIP".to_string(),
        score,
    );
    response.jwt = jwt;
    Json(response).into_response()
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
