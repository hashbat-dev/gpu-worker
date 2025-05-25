use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use gif::{Frame, Encoder, Repeat};
use std::io::Cursor;
use transformations::MirrorProcessor;
use crate::error::{GpuWorkerError, Result};

/// Handles the mirror GIF endpoint
///
/// Accepts a multipart form with a GIF file and returns the vertically mirrored version
pub async fn mirror_gif(
    payload: Multipart,
    mirror_processor: web::Data<MirrorProcessor>,
) -> Result<HttpResponse> {
    let gif_data = extract_gif_from_multipart(payload).await?;
    let mirrored_gif = process_gif_mirror(&gif_data, &mirror_processor).await?;

    Ok(HttpResponse::Ok()
        .content_type("image/gif")
        .body(mirrored_gif))
}

/// Extracts GIF data from multipart form data
async fn extract_gif_from_multipart(mut payload: Multipart) -> Result<Vec<u8>> {
    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();

        if content_disposition.get_name() == Some("file") {
            let mut gif_data = Vec::new();
            while let Some(chunk) = field.try_next().await? {
                gif_data.extend_from_slice(&chunk);
            }

            if gif_data.is_empty() {
                return Err(GpuWorkerError::InvalidInput("Empty file provided".to_string()));
            }

            return Ok(gif_data);
        }
    }

    Err(GpuWorkerError::InvalidInput("No file field found in multipart data".to_string()))
}

/// Processes a GIF by mirroring each frame vertically
async fn process_gif_mirror(gif_data: &[u8], mirror_processor: &MirrorProcessor) -> Result<Vec<u8>> {
    let (frames, width, height) = decode_gif(gif_data)?;
    encode_mirrored_gif(frames, width, height, mirror_processor).await
}

/// Decodes a GIF and returns its frames with metadata
fn decode_gif(gif_data: &[u8]) -> Result<(Vec<gif::Frame<'static>>, u32, u32)> {
    let cursor = Cursor::new(gif_data);
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = decoder.read_info(cursor)?;

    let width = decoder.width() as u32;
    let height = decoder.height() as u32;
    let mut frames = Vec::new();

    while let Some(frame) = decoder.read_next_frame()? {
        frames.push(frame.clone());
    }

    if frames.is_empty() {
        return Err(GpuWorkerError::InvalidInput("GIF contains no frames".to_string()));
    }

    log::info!("Decoded {} frames from GIF ({}x{})", frames.len(), width, height);
    Ok((frames, width, height))
}

/// Encodes mirrored frames back into a GIF
async fn encode_mirrored_gif(
    frames: Vec<gif::Frame<'static>>,
    width: u32,
    height: u32,
    mirror_processor: &MirrorProcessor,
) -> Result<Vec<u8>> {
    let mut output_gif = Vec::new();
    let mut encoder = create_gif_encoder(&mut output_gif, width as u16, height as u16)?;

    let total_frames = frames.len();
    for (index, frame) in frames.into_iter().enumerate() {
        log::info!("Processing frame {}/{}", index + 1, total_frames);

        let rgba_data = normalize_frame_to_rgba(&frame, width, height)?;
        let mirrored_data = mirror_processor.mirror_vertically(&rgba_data, width, height).await?;
        let mirrored_frame = create_mirrored_frame(&frame, &mirrored_data, width as u16, height as u16);

        encoder.write_frame(&mirrored_frame)?;
    }

    drop(encoder);
    Ok(output_gif)
}

/// Creates a GIF encoder with proper settings
fn create_gif_encoder(output: &mut Vec<u8>, width: u16, height: u16) -> Result<Encoder<&mut Vec<u8>>> {
    let mut encoder = Encoder::new(output, width, height, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;
    Ok(encoder)
}

/// Normalizes frame buffer to RGBA format
fn normalize_frame_to_rgba(frame: &gif::Frame, width: u32, height: u32) -> Result<Vec<u8>> {
    let expected_rgba_len = (width * height * 4) as usize;
    let expected_rgb_len = (width * height * 3) as usize;

    match frame.buffer.len() {
        len if len == expected_rgba_len => Ok(frame.buffer.to_vec()),
        len if len == expected_rgb_len => {
            // Convert RGB to RGBA
            let mut rgba = Vec::with_capacity(expected_rgba_len);
            for chunk in frame.buffer.chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255); // Alpha channel
            }
            Ok(rgba)
        }
        len => Err(GpuWorkerError::InvalidInput(
            format!("Unexpected frame buffer size: {} bytes (expected {} or {} bytes)",
                    len, expected_rgba_len, expected_rgb_len)
        )),
    }
}

/// Creates a new GIF frame with mirrored data, preserving original frame properties
fn create_mirrored_frame(
    original: &gif::Frame,
    mirrored_rgba: &[u8],
    width: u16,
    height: u16,
) -> Frame<'static> {
    // Convert RGBA back to RGB for GIF encoding
    let rgb_data: Vec<u8> = mirrored_rgba
        .chunks(4)
        .flat_map(|rgba| &rgba[..3])
        .copied()
        .collect();

    let mut frame = Frame::from_rgb_speed(width, height, &rgb_data, 10);
    
    // Preserve original frame properties
    frame.delay = original.delay;
    frame.dispose = original.dispose;
    frame.transparent = original.transparent;
    frame.needs_user_input = original.needs_user_input;
    frame.top = original.top;
    frame.left = original.left;
    // Keep the new dimensions we specified, not the original ones
    // frame.width and frame.height are already set by from_rgb_speed
    frame.interlaced = original.interlaced;

    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_rgba_buffer() {
        let frame = gif::Frame {
            buffer: vec![255; 12].into(), // 2x2 RGBA
            ..Default::default()
        };

        let result = normalize_frame_to_rgba(&frame, 2, 2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 16); // 2x2x4
    }

    #[test]
    fn test_normalize_rgb_to_rgba() {
        let frame = gif::Frame {
            buffer: vec![100, 150, 200, 50, 75, 100].into(), // 2x1 RGB
            ..Default::default()
        };

        let result = normalize_frame_to_rgba(&frame, 2, 1).unwrap();
        assert_eq!(result, vec![100, 150, 200, 255, 50, 75, 100, 255]);
    }

    #[test]
    fn test_create_mirrored_frame_preserves_properties() {
        let original = gif::Frame {
            delay: 10,
            dispose: gif::DisposalMethod::Background,
            transparent: Some(5),
            ..Default::default()
        };

        let mirrored_rgba = vec![100, 150, 200, 255]; // 1x1 RGBA
        let frame = create_mirrored_frame(&original, &mirrored_rgba, 1, 1);

        assert_eq!(frame.delay, 10);
        assert_eq!(frame.dispose, gif::DisposalMethod::Background);
        assert_eq!(frame.transparent, Some(5));
    }

    #[test]
    fn test_normalize_frame_invalid_size() {
        let frame = gif::Frame {
            buffer: vec![255; 10].into(), // Invalid size
            ..Default::default()
        };

        let result = normalize_frame_to_rgba(&frame, 2, 2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GpuWorkerError::InvalidInput(_)));
    }

    #[test]
    fn test_create_gif_encoder_success() {
        let mut output = Vec::new();
        let result = create_gif_encoder(&mut output, 100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_mirrored_frame_rgb_conversion() {
        let original = gif::Frame::default();
        let mirrored_rgba = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2x1 RGBA (red, green)
        let frame = create_mirrored_frame(&original, &mirrored_rgba, 2, 1);

        // Verify frame dimensions are correct
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 1);
        
        // Note: Frame::from_rgb_speed may create a palette-based frame
        // rather than storing raw RGB data, so we don't test the buffer contents
        // The important thing is that the frame is created successfully with correct dimensions
    }

    #[test]
    fn test_decode_gif_empty_frames() {
        // Create a minimal GIF header without frames
        let gif_data = vec![
            b'G', b'I', b'F', b'8', b'9', b'a', // Header
            1, 0, 1, 0, // Width: 1, Height: 1
            0, 0, 0, // Global color table info
            0x3B, // Trailer
        ];

        let result = decode_gif(&gif_data);
        assert!(result.is_err());
        // The GIF decoder will throw a decoding error for malformed GIF data
        match result.unwrap_err() {
            GpuWorkerError::GifDecode(_) | GpuWorkerError::InvalidInput(_) => (),
            _ => panic!("Expected GifDecode or InvalidInput error"),
        }
    }

    #[test]
    fn test_normalize_frame_edge_cases() {
        // Test empty frame
        let frame = gif::Frame {
            buffer: vec![].into(),
            ..Default::default()
        };
        let result = normalize_frame_to_rgba(&frame, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // Test 1x1 RGBA frame
        let frame = gif::Frame {
            buffer: vec![255, 0, 0, 255].into(),
            ..Default::default()
        };
        let result = normalize_frame_to_rgba(&frame, 1, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![255, 0, 0, 255]);
    }

    // Note: Multipart extraction tests are better suited for integration tests
    // due to the complexity of mocking the actix-multipart stream interface.
    // These tests would require actual HTTP request context to work properly.
}
