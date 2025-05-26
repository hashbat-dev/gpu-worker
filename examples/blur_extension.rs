use image::{ImageBuffer, Rgba};
use std::error::Error;
use tokio;
use transformations::BlurProcessor;

async fn process_image(
    path: &str,
    output_path: &str,
    blur_radius: f32,
) -> Result<(), Box<dyn Error>> {
    // Load the image
    println!("Loading image from: {}", path);
    let img = image::open(path)?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    println!("Image dimensions: {}x{}", width, height);

    // Initialize Blur processor
    println!("Initializing Blur processor...");
    let blur_processor = BlurProcessor::new().await?;

    // Process the image
    println!("Applying blur with radius: {}", blur_radius);
    let processed_data = blur_processor
        .blur_image(rgba_img.as_raw(), width, height, blur_radius)
        .await?;

    // Convert back to image and save
    let output_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, processed_data)
        .ok_or("Failed to create output image from processed data")?;

    output_img.save(output_path)?;
    println!("Processed image saved as: {}", output_path);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let (input_path, output_path, blur_radius) = match args.len() {
        1 => {
            // Default values for testing
            ("input.png", "output_blurred.png", 5.0)
        }
        2 => {
            // Input file provided
            (args[1].as_str(), "output_blurred.png", 5.0)
        }
        3 => {
            // Input and output files provided
            (args[1].as_str(), args[2].as_str(), 5.0)
        }
        4 => {
            // All parameters provided
            let radius = args[3].parse::<f32>().unwrap_or_else(|_| {
                eprintln!("Invalid blur radius, using default: 5.0");
                5.0
            });
            (args[1].as_str(), args[2].as_str(), radius)
        }
        _ => {
            eprintln!(
                "Usage: {} [input_image] [output_image] [blur_radius]",
                args[0]
            );
            eprintln!("Example: {} image.png blurred.png 10.0", args[0]);
            return Ok(());
        }
    };

    process_image(input_path, output_path, blur_radius).await?;

    Ok(())
}
