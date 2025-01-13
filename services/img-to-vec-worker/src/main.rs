#![allow(clippy::redundant_pub_crate)]

use anyhow::{Context, Error};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::Qdrant;

use xlib::client::{PostgresClient, PostgresClientConfig};

use core::fmt;
use thiserror::Error;
use tracing::{info, warn};

use reqwest::header;
use serde::{Deserialize, Serialize};

use base64;
use std::{env, str::FromStr, time::Duration};
async fn start_background_worker() {
    let qdrantClient = Qdrant::from_url("http://qdrant:6334").build().unwrap();
    let collection_name = "clip_images_collection";
    qdrantClient
        .create_collection(
            CreateCollectionBuilder::new(collection_name)
                .vectors_config(VectorParamsBuilder::new(4, Distance::Cosine)),
        )
        .await
        .unwrap();
    info!("Collection created");
    let points = vec![
        PointStruct::new(1, vec![0.05, 0.61, 0.76, 0.74], [("city", "Berlin".into())]),
        PointStruct::new(2, vec![0.19, 0.81, 0.75, 0.11], [("city", "London".into())]),
        PointStruct::new(3, vec![0.36, 0.55, 0.47, 0.94], [("city", "Moscow".into())]),
        // ..truncated
    ];

    let HttpClient = reqwest::Client::builder().build().unwrap();

    // let db_client = init_db().await;
    let mut completed_images = Vec::new();

    // Set up graceful shutdown channel
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Handle Ctrl+C
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await {
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
                                        let base64_image = base64::encode(&image_data);
                                        info!("Generated base64 for image: {}", file_name);
                                        let data = serde_json::json!({
                                            "image_base64": base64_image
                                        });

                                        match HttpClient.request(reqwest::Method::POST, "http://clip-model:8000/api/v1/clip/image-to-vector")
                                            .json(&data)
                                            .send()
                                            .await {
                                                Ok(response) => {
                                                    match response.text().await {
                                                        Ok(body) => {
                                                            info!("Got vector response for image: {}", file_name);
                                                            // TODO: Store vector in Qdrant
                                                            info!("Response body: {}", body);

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
