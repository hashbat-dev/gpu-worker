# Transformations Crate

A GPU-accelerated image transformation library using WebGPU (wgpu) for high-performance image processing operations.

## Features

- **Blur Transformation**: Apply box blur effects with configurable radius
- **Mirror Transformation**: Vertically mirror/flip images
- **GPU Acceleration**: Leverages WebGPU for fast parallel processing
- **Async API**: Built with async/await for non-blocking operations
- **Memory Efficient**: Handles padding and buffer alignment automatically

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
transformations = { path = "transformations" }
```

## Usage

### Blur Transformation

```rust
use transformations::BlurProcessor;
use image::open;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the blur processor
    let blur_processor = BlurProcessor::new().await?;
    
    // Load an image
    let img = open("input.png")?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    // Apply blur with radius 5.0
    let blurred_data = blur_processor.blur_image(
        rgba_img.as_raw(),
        width,
        height,
        5.0
    ).await?;
    
    // Save the result
    let output_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, blurred_data)
        .ok_or("Failed to create output image")?;
    output_img.save("blurred.png")?;
    
    Ok(())
}
```

### Mirror Transformation

```rust
use transformations::MirrorProcessor;
use image::open;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the mirror processor
    let mirror_processor = MirrorProcessor::new().await?;
    
    // Load an image
    let img = open("input.png")?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    // Apply vertical mirror
    let mirrored_data = mirror_processor.mirror_vertically(
        rgba_img.as_raw(),
        width,
        height
    ).await?;
    
    // Save the result
    let output_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, mirrored_data)
        .ok_or("Failed to create output image")?;
    output_img.save("mirrored.png")?;
    
    Ok(())
}
```

## API Reference

### BlurProcessor

#### `BlurProcessor::new() -> Result<Self>`
Creates a new blur processor instance with GPU initialization.

#### `blur_image(&self, image_data: &[u8], width: u32, height: u32, blur_radius: f32) -> Result<Vec<u8>>`
Applies a box blur to the input image data.

- `image_data`: Raw RGBA image bytes
- `width`: Image width in pixels
- `height`: Image height in pixels
- `blur_radius`: Blur radius (higher values = more blur)

### MirrorProcessor

#### `MirrorProcessor::new() -> Result<Self>`
Creates a new mirror processor instance with GPU initialization.

#### `mirror_vertically(&self, image_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>>`
Mirrors the image vertically (flips upside down).

- `image_data`: Raw RGBA image bytes
- `width`: Image width in pixels
- `height`: Image height in pixels

## Error Handling

The crate uses a custom `TransformationError` type that covers:

- GPU initialization errors
- Image processing errors
- Buffer operations errors
- Invalid input errors

## Examples

See the `examples/` directory in the parent project:

- `blur_extension.rs`: Command-line blur tool
- `mirror_example.rs`: Command-line mirror tool

Run examples with:

```bash
cargo run --example blur_extension input.png output.png 10.0
cargo run --example mirror_example input.png output.png
```

## Performance

The transformations leverage GPU parallelization for optimal performance:

- Blur operations process multiple pixels simultaneously
- Mirror transformations use GPU texture sampling
- Automatic buffer alignment for efficient GPU memory access

## Requirements

- Rust 1.70+
- WebGPU-compatible GPU
- wgpu 0.19

## License

See the parent project's license file.