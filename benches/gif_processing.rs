use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gpu_worker::handlers::mirror_gif;
use actix_web::{web, test, App};
use actix_multipart::test::create_stream;
use bytes::Bytes;
use futures_util::stream;
use transformations::MirrorProcessor;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn create_test_gif(width: u16, height: u16, frames: u8) -> Vec<u8> {
    use gif::{Encoder, Frame, Repeat};
    use std::io::Cursor;
    
    let mut gif_data = Vec::new();
    let mut encoder = Encoder::new(&mut gif_data, width, height, &[]).unwrap();
    encoder.set_repeat(Repeat::Finite(0)).unwrap();
    
    // Create a simple pattern for each frame
    for frame_num in 0..frames {
        let mut pixels = vec![0u8; (width as usize * height as usize * 3)];
        
        // Create a gradient pattern that changes with each frame
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                pixels[idx] = ((x as f32 / width as f32) * 255.0) as u8;
                pixels[idx + 1] = ((y as f32 / height as f32) * 255.0) as u8;
                pixels[idx + 2] = frame_num.wrapping_mul(30);
            }
        }
        
        let frame = Frame::from_rgb_speed(width, height, &pixels, 10);
        encoder.write_frame(&frame).unwrap();
    }
    
    drop(encoder);
    gif_data
}

fn create_multipart_payload(gif_data: Vec<u8>) -> Vec<u8> {
    let boundary = "----boundary----";
    let mut data = Vec::new();
    
    data.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    data.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"test.gif\"\r\n");
    data.extend_from_slice(b"Content-Type: image/gif\r\n\r\n");
    data.extend_from_slice(&gif_data);
    data.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());
    
    data
}

fn bench_gif_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("gif_sizes");
    
    let sizes = vec![
        ("small", 100, 100, 5),
        ("medium", 500, 500, 5),
        ("large", 1000, 1000, 5),
        ("hd", 1920, 1080, 5),
    ];
    
    for (name, width, height, frames) in sizes {
        let gif_data = create_test_gif(width, height, frames);
        let payload = create_multipart_payload(gif_data);
        
        group.bench_with_input(
            BenchmarkId::new("process", name),
            &payload,
            |b, payload| {
                b.to_async(&rt).iter(|| async {
                    let mirror_processor = MirrorProcessor::new().await.unwrap();
                    let app = test::init_service(
                        App::new()
                            .app_data(web::Data::new(mirror_processor))
                            .route("/mirror-gif", web::post().to(mirror_gif))
                    ).await;
                    
                    let req = test::TestRequest::post()
                        .uri("/mirror-gif")
                        .insert_header(("content-type", "multipart/form-data; boundary=----boundary----"))
                        .set_payload(payload.clone())
                        .to_request();
                    
                    let _resp = test::call_service(&app, req).await;
                });
            },
        );
    }
    
    group.finish();
}

fn bench_frame_counts(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("frame_counts");
    
    let frame_counts = vec![1, 5, 10, 20, 50];
    
    for frames in frame_counts {
        let gif_data = create_test_gif(500, 500, frames);
        let payload = create_multipart_payload(gif_data);
        
        group.bench_with_input(
            BenchmarkId::new("frames", frames),
            &payload,
            |b, payload| {
                b.to_async(&rt).iter(|| async {
                    let mirror_processor = MirrorProcessor::new().await.unwrap();
                    let app = test::init_service(
                        App::new()
                            .app_data(web::Data::new(mirror_processor))
                            .route("/mirror-gif", web::post().to(mirror_gif))
                    ).await;
                    
                    let req = test::TestRequest::post()
                        .uri("/mirror-gif")
                        .insert_header(("content-type", "multipart/form-data; boundary=----boundary----"))
                        .set_payload(payload.clone())
                        .to_request();
                    
                    let _resp = test::call_service(&app, req).await;
                });
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_requests");
    
    let gif_data = create_test_gif(200, 200, 3);
    let payload = create_multipart_payload(gif_data);
    
    for concurrency in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrency", concurrency),
            &payload,
            |b, payload| {
                b.to_async(&rt).iter(|| async {
                    let mirror_processor = Arc::new(MirrorProcessor::new().await.unwrap());
                    let app = Arc::new(test::init_service(
                        App::new()
                            .app_data(web::Data::new((*mirror_processor).clone()))
                            .route("/mirror-gif", web::post().to(mirror_gif))
                    ).await);
                    
                    let mut handles = vec![];
                    
                    for _ in 0..*concurrency {
                        let app_clone = app.clone();
                        let payload_clone = payload.clone();
                        
                        let handle = tokio::spawn(async move {
                            let req = test::TestRequest::post()
                                .uri("/mirror-gif")
                                .insert_header(("content-type", "multipart/form-data; boundary=----boundary----"))
                                .set_payload(payload_clone)
                                .to_request();
                            
                            test::call_service(&*app_clone, req).await
                        });
                        
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        let _ = handle.await.unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_gpu_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("gpu_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = black_box(MirrorProcessor::new().await.unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_gif_sizes,
    bench_frame_counts,
    bench_concurrent_requests,
    bench_gpu_initialization
);
criterion_main!(benches);