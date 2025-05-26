//! # GPU Worker
//!
//! A high-performance GPU-accelerated microservice for image processing operations.
//!
//! This crate provides a REST API service that leverages WebGPU for fast image transformations,
//! with a focus on GIF processing. The service is built on Actix-web for high concurrency
//! and uses the `transformations` crate for GPU operations.
//!
//! ## Features
//!
//! - **GPU Acceleration**: Uses WebGPU for hardware-accelerated image processing
//! - **GIF Support**: Full support for animated GIF processing
//! - **RESTful API**: Simple HTTP endpoints for easy integration
//! - **Async Processing**: Non-blocking request handling for high throughput
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//!
//! - [`error`]: Error types and HTTP error responses
//! - [`handlers`]: HTTP request handlers for API endpoints
//!
//! ## Example
//!
//! ```no_run
//! use actix_web::{web, App, HttpServer};
//! use gpu_worker::handlers::mirror_gif;
//! use transformations::MirrorProcessor;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let mirror_processor = web::Data::new(
//!         MirrorProcessor::new().await.expect("Failed to create MirrorProcessor")
//!     );
//!     
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(mirror_processor.clone())
//!             .route("/mirror-gif", web::post().to(mirror_gif))
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```

pub mod error;
pub mod handlers;

pub use error::{GpuWorkerError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_exists() {
        let _error = GpuWorkerError::InvalidInput("test".to_string());
    }
}
