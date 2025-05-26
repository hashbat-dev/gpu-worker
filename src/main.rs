use actix_web::{middleware, web, App, HttpServer};
use log::info;
use std::sync::Arc;

mod error;
mod handlers;

use handlers::mirror_gif;
use transformations::MirrorProcessor;

#[derive(Clone)]
struct AppState {
    mirror_processor: Arc<MirrorProcessor>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let config = Config::from_env();

    info!("Initializing GPU Worker microservice");
    info!("Server configuration: {:?}", config);

    let app_state = initialize_app_state().await?;

    info!("Starting server on {}:{}", config.host, config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .route("/mirror-gif", web::post().to(mirror_gif_handler)),
            )
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif_handler))
    })
    .bind((config.host.as_str(), config.port))?
    .workers(config.workers)
    .run()
    .await
}

async fn initialize_app_state() -> std::io::Result<AppState> {
    info!("Initializing Mirror processor...");

    let mirror_processor = MirrorProcessor::new().await.map_err(|e| {
        log::error!("Failed to create MirrorProcessor: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    Ok(AppState {
        mirror_processor: Arc::new(mirror_processor),
    })
}

async fn mirror_gif_handler(
    payload: actix_multipart::Multipart,
    app_state: web::Data<AppState>,
) -> Result<impl actix_web::Responder, error::GpuWorkerError> {
    mirror_gif(payload, web::Data::from(app_state.mirror_processor.clone())).await
}

async fn health_check() -> actix_web::Result<impl actix_web::Responder> {
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        service: "gpu-worker".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        features: vec!["mirror-gif".to_string()],
    };

    Ok(web::Json(health_status))
}

#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
    service: String,
    version: String,
    features: Vec<String>,
}

#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    workers: usize,
}

impl Config {
    fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            workers: std::env::var("WORKERS")
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or_else(num_cpus::get),
        }
    }
}

fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::from_env();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.workers > 0);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            status: "healthy".to_string(),
            service: "test".to_string(),
            version: "1.0.0".to_string(),
            features: vec!["feature1".to_string()],
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"service\":\"test\""));
    }
}
