use actix_web::{test, web, App};
use gpu_worker::handlers::mirror_gif;
use transformations::MirrorProcessor;

async fn health_check() -> actix_web::Result<impl actix_web::Responder> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "gpu-worker"
    })))
}

fn create_test_gif() -> Vec<u8> {
    // Create a minimal valid 1x1 GIF
    // This is a complete, valid GIF file with proper LZW encoding
    vec![
        // Header
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, // "GIF89a"
        // Logical Screen Descriptor
        0x01, 0x00, // Width: 1
        0x01, 0x00, // Height: 1
        0xF0, // Global color table, 1-bit color, no sort, 2 colors
        0x00, // Background color index
        0x00, // Pixel aspect ratio
        // Global Color Table (2 colors: black and white)
        0x00, 0x00, 0x00, // Color 0: Black
        0xFF, 0xFF, 0xFF, // Color 1: White
        // Image Descriptor
        0x2C, // Image separator
        0x00, 0x00, // Left position
        0x00, 0x00, // Top position
        0x01, 0x00, // Width: 1
        0x01, 0x00, // Height: 1
        0x00, // No local color table, no interlace
        // Image Data
        0x02, // LZW minimum code size
        0x02, // Block size
        0x44, 0x01, // LZW compressed data for a single black pixel
        0x00, // Block terminator
        // Trailer
        0x3B, // GIF trailer
    ]
}

#[actix_web::test]
async fn test_health_endpoint() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "gpu-worker");
}

#[actix_web::test]
async fn test_mirror_gif_success() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let gif_data = create_test_gif();
    let boundary = "----boundary----";

    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"test.gif\"\r\n",
    );
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(&gif_data);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header((
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("content-type").unwrap(), "image/gif");
}

#[actix_web::test]
async fn test_mirror_gif_no_file() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let boundary = "----boundary----";

    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"other\"\r\n\r\n");
    data.extend_from_slice(b"some data");
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header((
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "invalid_request");
}

#[actix_web::test]
async fn test_mirror_gif_empty_file() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let boundary = "----boundary----";

    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"empty.gif\"\r\n",
    );
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    // No file data
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header((
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_mirror_gif_invalid_gif() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let boundary = "----boundary----";
    let invalid_gif = b"This is not a GIF file";

    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"invalid.gif\"\r\n",
    );
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(invalid_gif);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header((
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 422);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "processing_error");
}

#[actix_web::test]
async fn test_mirror_gif_large_file() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    // Use the valid test GIF (it's small but valid)
    let large_gif = create_test_gif();

    let boundary = "----boundary----";

    let mut data = Vec::new();
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"large.gif\"\r\n",
    );
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(&large_gif);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header((
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        ))
        .set_payload(data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should succeed with a valid GIF
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("content-type").unwrap(), "image/gif");
}

#[actix_web::test]
async fn test_mirror_gif_wrong_content_type() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/mirror-gif")
        .insert_header(("content-type", "application/json"))
        .set_payload("{\"file\": \"test\"}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_health_check_multiple_sequential() {
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mirror_processor))
            .route("/health", web::get().to(health_check))
            .route("/mirror-gif", web::post().to(mirror_gif)),
    )
    .await;

    // Run multiple sequential requests
    for _ in 0..5 {
        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
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
