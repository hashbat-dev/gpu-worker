use actix_web::{test, web, App};
use actix_multipart::test::create_stream;
use actix_multipart::test::create_stream_with_headers;
use bytes::Bytes;
use futures_util::stream;
use gpu_worker::handlers::mirror_gif;
use transformations::MirrorProcessor;
use std::sync::Arc;

async fn create_test_app() -> App {

    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");
    
    App::new()
        .app_data(web::Data::new(mirror_processor))
        .route("/health", web::get().to(health_check))
        .route("/mirror-gif", web::post().to(mirror_gif))
}

async fn health_check() -> actix_web::Result<impl actix_web::Responder> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "gpu-worker"
    })))
}

fn create_test_gif() -> Vec<u8> {
    // Create a minimal valid GIF
    let header = b"GIF89a";
    let logical_screen = [
        10, 0,  // Width: 10
        10, 0,  // Height: 10
        0x80,   // Global color table flag
        0,      // Background color index
        0,      // Pixel aspect ratio
    ];
    let global_color_table = [
        0, 0, 0,       // Black
        255, 255, 255, // White
    ];
    let image_descriptor = [
        0x2C,   // Image separator
        0, 0,   // Left position
        0, 0,   // Top position
        10, 0,  // Width
        10, 0,  // Height
        0,      // No local color table
    ];
    let image_data = [
        2,      // LZW minimum code size
        1,      // Block size
        0,      // Data
        0,      // Block terminator
    ];
    let trailer = [0x3B]; // GIF trailer

    let mut gif = Vec::new();
    gif.extend_from_slice(header);
    gif.extend_from_slice(&logical_screen);
    gif.extend_from_slice(&global_color_table);
    gif.extend_from_slice(&image_descriptor);
    gif.extend_from_slice(&image_data);
    gif.extend_from_slice(&trailer);
    gif
}

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(create_test_app().await).await;
    
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "gpu-worker");
}

#[actix_web::test]
async fn test_mirror_gif_success() {
    let app = test::init_service(create_test_app().await).await;
    
    let gif_data = create_test_gif();
    let boundary = "----boundary----";
    
    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"test.gif\"\r\n");
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(&gif_data);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", format!("multipart/form-data; boundary={}", boundary)))
        .set_payload(data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("content-type").unwrap(), "image/gif");
}

#[actix_web::test]
async fn test_mirror_gif_no_file() {
    let app = test::init_service(create_test_app().await).await;
    
    let boundary = "----boundary----";
    
    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"other\"\r\n\r\n");
    data.extend_from_slice(b"some data");
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", format!("multipart/form-data; boundary={}", boundary)))
        .set_payload(data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "invalid_request");
}

#[actix_web::test]
async fn test_mirror_gif_empty_file() {
    let app = test::init_service(create_test_app().await).await;
    
    let boundary = "----boundary----";
    
    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"empty.gif\"\r\n");
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    // No file data
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", format!("multipart/form-data; boundary={}", boundary)))
        .set_payload(data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_mirror_gif_invalid_gif() {
    let app = test::init_service(create_test_app().await).await;
    
    let boundary = "----boundary----";
    let invalid_gif = b"This is not a GIF file";
    
    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"invalid.gif\"\r\n");
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(invalid_gif);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", format!("multipart/form-data; boundary={}", boundary)))
        .set_payload(data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 422);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "processing_error");
}

#[actix_web::test]
async fn test_mirror_gif_large_file() {
    let app = test::init_service(create_test_app().await).await;
    
    // Create a larger test GIF (still small but with more data)
    let mut large_gif = create_test_gif();
    for _ in 0..100 {
        large_gif.extend_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
    
    let boundary = "----boundary----";
    
    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"large.gif\"\r\n");
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(&large_gif);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", format!("multipart/form-data; boundary={}", boundary)))
        .set_payload(data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    // This should fail because our extended data makes an invalid GIF
    assert_eq!(resp.status(), 422);
}

#[actix_web::test]
async fn test_mirror_gif_wrong_content_type() {
    let app = test::init_service(create_test_app().await).await;
    
    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", "application/json"))
        .set_payload("{\"file\": \"test\"}")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_concurrent_requests() {
    let app = test::init_service(create_test_app().await).await;
    let app = Arc::new(app);
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let req = test::TestRequest::get()
                .uri("/health")
                .to_request();
            
            let resp = test::call_service(&*app_clone, req).await;
            assert!(resp.status().is_success());
            i
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await.unwrap();
    }
}

#[cfg(test)]
mod unit_tests {
    use gpu_worker::error::GpuWorkerError;
    
    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let gpu_error: GpuWorkerError = io_error.into();
        assert!(matches!(gpu_error, GpuWorkerError::Io(_)));
    }
    
    #[test]
    fn test_error_display() {
        let error = GpuWorkerError::InvalidInput("test input".to_string());
        assert_eq!(error.to_string(), "Invalid input: test input");
        
        let error = GpuWorkerError::Gpu("GPU failed".to_string());
        assert_eq!(error.to_string(), "GPU error: GPU failed");
    }
}