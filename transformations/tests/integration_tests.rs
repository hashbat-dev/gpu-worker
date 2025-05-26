use image::{ImageBuffer, Rgba};
use transformations::{BlurProcessor, MirrorProcessor};

// Helper function to create a test image
fn create_test_image(width: u32, height: u32) -> Vec<u8> {
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    // Create a simple gradient pattern
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = 128;
        let a = 255;
        *pixel = Rgba([r, g, b, a]);
    }

    image.into_raw()
}

#[tokio::test]
async fn test_blur_processor_creation() {
    let processor = BlurProcessor::new().await;
    assert!(
        processor.is_ok(),
        "Failed to create BlurProcessor: {:?}",
        processor.err()
    );
}

#[tokio::test]
async fn test_mirror_processor_creation() {
    let processor = MirrorProcessor::new().await;
    assert!(
        processor.is_ok(),
        "Failed to create MirrorProcessor: {:?}",
        processor.err()
    );
}

#[tokio::test]
async fn test_blur_image_processing() {
    let processor = BlurProcessor::new()
        .await
        .expect("Failed to create BlurProcessor");

    let width = 100;
    let height = 100;
    let test_image = create_test_image(width, height);
    let blur_radius = 5.0;

    let result = processor
        .blur_image(&test_image, width, height, blur_radius)
        .await;

    assert!(result.is_ok(), "Blur processing failed: {:?}", result.err());

    let processed_data = result.unwrap();
    assert_eq!(
        processed_data.len(),
        (width * height * 4) as usize,
        "Output image has incorrect size"
    );
}

#[tokio::test]
async fn test_mirror_image_processing() {
    let processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let width = 100;
    let height = 100;
    let test_image = create_test_image(width, height);

    let result = processor
        .mirror_vertically(&test_image, width, height)
        .await;

    assert!(
        result.is_ok(),
        "Mirror processing failed: {:?}",
        result.err()
    );

    let processed_data = result.unwrap();
    assert_eq!(
        processed_data.len(),
        (width * height * 4) as usize,
        "Output image has incorrect size"
    );
}

#[tokio::test]
async fn test_blur_with_different_radii() {
    let processor = BlurProcessor::new()
        .await
        .expect("Failed to create BlurProcessor");

    let width = 50;
    let height = 50;
    let test_image = create_test_image(width, height);

    // Test with different blur radii
    let radii = vec![0.0, 1.0, 3.0, 5.0, 10.0];

    for radius in radii {
        let result = processor
            .blur_image(&test_image, width, height, radius)
            .await;
        assert!(
            result.is_ok(),
            "Blur processing failed with radius {}: {:?}",
            radius,
            result.err()
        );
    }
}

#[tokio::test]
async fn test_mirror_image_content() {
    let processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let width = 10;
    let height = 10;

    // Create a simple asymmetric image
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    // Fill top half with white, bottom half with black
    for y in 0..height {
        for x in 0..width {
            let color = if y < height / 2 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            };
            image.put_pixel(x, y, color);
        }
    }

    let input_data = image.clone().into_raw();
    let result = processor
        .mirror_vertically(&input_data, width, height)
        .await
        .unwrap();

    // Convert result back to image
    let output_image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, result)
        .expect("Failed to create output image");

    // Verify that the image is mirrored
    // Top should now be black, bottom should be white
    let top_pixel = output_image.get_pixel(width / 2, 0);
    let bottom_pixel = output_image.get_pixel(width / 2, height - 1);

    assert_eq!(top_pixel[0], 0, "Top pixel should be black after mirroring");
    assert_eq!(
        bottom_pixel[0], 255,
        "Bottom pixel should be white after mirroring"
    );
}

#[tokio::test]
async fn test_large_image_processing() {
    let blur_processor = BlurProcessor::new()
        .await
        .expect("Failed to create BlurProcessor");
    let mirror_processor = MirrorProcessor::new()
        .await
        .expect("Failed to create MirrorProcessor");

    let width = 1920;
    let height = 1080;
    let test_image = create_test_image(width, height);

    // Test blur on large image
    let blur_result = blur_processor
        .blur_image(&test_image, width, height, 3.0)
        .await;
    assert!(
        blur_result.is_ok(),
        "Failed to blur large image: {:?}",
        blur_result.err()
    );

    // Test mirror on large image
    let mirror_result = mirror_processor
        .mirror_vertically(&test_image, width, height)
        .await;
    assert!(
        mirror_result.is_ok(),
        "Failed to mirror large image: {:?}",
        mirror_result.err()
    );
}

#[tokio::test]
async fn test_single_pixel_image() {
    let processor = BlurProcessor::new()
        .await
        .expect("Failed to create BlurProcessor");

    // Test with single pixel
    let width = 1;
    let height = 1;
    let test_image = vec![255, 0, 0, 255]; // Red pixel

    let result = processor.blur_image(&test_image, width, height, 5.0).await;

    // Single pixel blur should succeed
    assert!(
        result.is_ok(),
        "Failed to blur single pixel image: {:?}",
        result.err()
    );

    let processed_data = result.unwrap();
    assert_eq!(
        processed_data.len(),
        4,
        "Single pixel output should be 4 bytes"
    );
}
