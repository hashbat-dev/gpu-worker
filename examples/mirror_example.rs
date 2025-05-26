//! Example demonstrating how to use the GPU Worker service to mirror GIF images
//!
//! This example shows how to:
//! - Send a GIF file to the mirror endpoint
//! - Handle the response
//! - Save the mirrored GIF to disk
//!
//! Usage:
//! ```bash
//! cargo run --example mirror_example -- input.gif output.gif
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::multipart;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Mirror a GIF using the GPU Worker service")]
struct Args {
    /// Input GIF file path
    #[clap(value_parser)]
    input: PathBuf,

    /// Output GIF file path
    #[clap(value_parser)]
    output: PathBuf,

    /// GPU Worker service URL
    #[clap(short, long, default_value = "http://localhost:8080")]
    url: String,

    /// Show detailed timing information
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Validate input file
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {:?}", args.input);
    }

    if !args.input.extension().is_some_and(|ext| ext == "gif") {
        eprintln!("Warning: Input file does not have .gif extension");
    }

    // Read the input file
    let start = Instant::now();
    let file_data = fs::read(&args.input)
        .with_context(|| format!("Failed to read input file: {:?}", args.input))?;

    let file_size = file_data.len();
    println!("Read {} bytes from {:?}", file_size, args.input);

    if args.verbose {
        println!("File read took: {:?}", start.elapsed());
    }

    // Create the multipart form
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_data)
            .file_name(
                args.input
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
            .mime_str("image/gif")?,
    );

    // Send the request
    let client = reqwest::Client::new();
    let url = format!("{}/mirror-gif", args.url);

    println!("Sending request to {}", url);
    let request_start = Instant::now();

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .with_context(|| format!("Failed to send request to {}", url))?;

    if args.verbose {
        println!("Request took: {:?}", request_start.elapsed());
        println!("Response status: {}", response.status());
        println!("Response headers: {:?}", response.headers());
    }

    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Server returned error {}: {}", status, error_body);
    }

    // Get the response body
    let download_start = Instant::now();
    let response_bytes = response
        .bytes()
        .await
        .context("Failed to download response body")?;

    if args.verbose {
        println!("Download took: {:?}", download_start.elapsed());
        println!("Response size: {} bytes", response_bytes.len());
    }

    // Save the output file
    let save_start = Instant::now();
    let mut output_file = File::create(&args.output)
        .await
        .with_context(|| format!("Failed to create output file: {:?}", args.output))?;

    output_file
        .write_all(&response_bytes)
        .await
        .with_context(|| format!("Failed to write output file: {:?}", args.output))?;

    output_file
        .flush()
        .await
        .context("Failed to flush output file")?;

    if args.verbose {
        println!("Save took: {:?}", save_start.elapsed());
    }

    // Print summary
    println!("\nSuccess! Mirrored GIF saved to {:?}", args.output);
    println!("Input size:  {} bytes", file_size);
    println!("Output size: {} bytes", response_bytes.len());
    println!("Total time:  {:?}", start.elapsed());

    if args.verbose {
        let size_ratio = response_bytes.len() as f64 / file_size as f64;
        println!("Size ratio:  {:.2}x", size_ratio);

        let processing_time = request_start.elapsed() - download_start.elapsed();
        println!("Server processing time: ~{:?}", processing_time);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&[
            "mirror_example",
            "input.gif",
            "output.gif",
            "--url",
            "http://example.com",
            "--verbose",
        ]);

        assert_eq!(args.input, PathBuf::from("input.gif"));
        assert_eq!(args.output, PathBuf::from("output.gif"));
        assert_eq!(args.url, "http://example.com");
        assert!(args.verbose);
    }

    #[test]
    fn test_default_url() {
        let args = Args::parse_from(&["mirror_example", "input.gif", "output.gif"]);

        assert_eq!(args.url, "http://localhost:8080");
    }
}
