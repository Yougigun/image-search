#![allow(clippy::redundant_pub_crate)]

use anyhow::{Context, Error};

use aws_sdk_s3::Client;

use xlib::client::{PostgresClient, PostgresClientConfig};

use core::fmt;
use thiserror::Error;
use tracing::{info, warn};

use reqwest::header;
use serde::{Deserialize, Serialize};

use std::{env, str::FromStr, time::Duration};
use uuid::Uuid;

async fn start_background_worker() {
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
