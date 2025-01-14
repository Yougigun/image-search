#![allow(clippy::redundant_pub_crate)]

use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::Qdrant;

use base64::Engine;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;
async fn start_background_worker() {
    let qdrant_client = Qdrant::from_url("http://qdrant:6334").build().unwrap();
    let collection_name = "clip_images_collection";
    let vector_size = 512;
    // Check if collection exists first
    if let Ok(collections) = qdrant_client.list_collections().await {
        if !collections
            .collections
            .iter()
            .any(|c| c.name == collection_name)
        {
            // Collection doesn't exist, create it
            if let Err(e) = qdrant_client
                .create_collection(
                    CreateCollectionBuilder::new(collection_name)
                        .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
                )
                .await
            {
                warn!("Failed to create collection: {}", e);
            } else {
                info!("Created new collection: {}", collection_name);
            }
        } else {
            info!("Collection {} already exists", collection_name);
        }
    } else {
        warn!("Failed to check existing collections");
    }
    info!("Collection created");

    let http_client = reqwest::Client::builder().build().unwrap();

    // In MVP we only deploy one container but in the future we might need to scale up the worker
    // so we should use external database to record the processed images
    let mut completed_images = Vec::new();

    // Set up graceful shutdown channel
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Handle Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received shutdown signal");
            let _ = shutdown_tx_clone.send(());
        }
    });

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Shutting down worker gracefully...");
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                match std::fs::read_dir("/images") {
                    Ok(entries) => {
                        for (i, entry) in entries.flatten().enumerate() {
                            if let Ok(file_name) = entry.file_name().into_string() {
                                // check if the image is already processed
                                if completed_images.contains(&file_name) {
                                    continue;
                                } else {
                                    info!("Found un processed image file #{}: {}", i, file_name);
                                    // TODO: process the image
                                    let image_path = entry.path();
                                    if let Ok(image_data) = std::fs::read(&image_path) {
                                        // let base64_image = base64::encode(&image_data);
                                        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
                                        info!("Generated base64 for image: {}", file_name);
                                        let data = serde_json::json!({
                                            "image_base64": base64_image
                                        });

                                        match http_client.request(reqwest::Method::POST, "http://clip-model:8000/api/v1/clip/image-to-vector")
                                            .json(&data)
                                            .send()
                                            .await {
                                                Ok(response) => {
                                                    match response.text().await {
                                                        Ok(body) => {
                                                            info!("Got vector response for image: {}", file_name);
                                                            // Parse the response body as JSON
                                                            if let Ok(vector_response) = serde_json::from_str::<serde_json::Value>(&body) {
                                                                if let Some(vector) = vector_response.get("vector") {
                                                                    if let Some(vector_array) = vector.as_array() {
                                                                        // Convert JSON array to Vec<f32>
                                                                        let vector_data: Vec<f32> = vector_array
                                                                            .iter()
                                                                            .filter_map(|v| v.as_f64().map(|x| x as f32))
                                                                            .collect();

                                                                        info!("Successfully parsed vector with {} dimensions", vector_data.len());
                                                                        let point = PointStruct::new(
                                                                            Uuid::new_v5(&Uuid::NAMESPACE_URL, file_name.as_bytes()).to_string(),
                                                                            vector_data,
                                                                            [("image_name", file_name.clone().into())]
                                                                        );
                                                                        qdrant_client.upsert_points(UpsertPointsBuilder::new(collection_name, vec![point]).wait(true)).await.unwrap();
                                                                        info!("Upserted point into Qdrant");
                                                                    } else {
                                                                        warn!("Vector field is not an array");
                                                                    }
                                                                } else {
                                                                    warn!("Response missing vector field");
                                                                }
                                                            } else {
                                                                warn!("Failed to parse response as JSON: {}", body);
                                                            }

                                                        }
                                                        Err(e) => {
                                                            warn!("Failed to get response body: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!("Failed to send request to CLIP model: {}", e);
                                                }
                                        };


                                    } else {
                                        warn!("Failed to read image file: {}", file_name);
                                        continue;
                                    }
                                    completed_images.push(file_name);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error reading /images directory: {}", e);
                    }
                }
            }
        }
    }

    info!("Worker shutdown complete");
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stdout)
        .init();
    // Log when the program starts
    info!("Starting img-to-vec worker...");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_background_worker());
}
