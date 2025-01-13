use std::net::SocketAddrV4;

use anyhow::{Context, Result};
use axum::Router;

use crate::app::graceful_shutdown;

pub async fn serve_service(
    router: Router,
    addr: SocketAddrV4,
    service_name: &'static str,
) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Service `{service_name}`: could not listen on {addr}"))?;

    tracing::info!("Service `{service_name}`: listening on {addr:?}");

    axum::serve(listener, router)
        .with_graceful_shutdown(graceful_shutdown::shutdown_signal())
        .await
        .with_context(|| format!("Service `{service_name}`: failed to serve"))
}
