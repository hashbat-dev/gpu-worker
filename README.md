# GPU Worker Microservice

A high-performance GPU-accelerated microservice for image processing operations, specifically designed for GIF transformations using WebGPU.

## Features

- **GPU Acceleration**: Leverages WebGPU for fast image processing
- **GIF Support**: Full support for animated GIF processing
- **Mirror Transformation**: Vertical mirroring of GIF images
- **RESTful API**: Simple HTTP API for easy integration
- **Async Processing**: Built on Actix-web for high concurrency
- **Health Monitoring**: Built-in health check endpoint
- **Configurable**: Environment-based configuration

## Requirements

- Rust 1.70+ (for async traits)
- GPU with WebGPU support
- Linux, macOS, or Windows

## Installation

### From Source

```bash
git clone https://github.com/yourusername/gpu-worker.git
cd gpu-worker
cargo build --release
```

### Docker

```bash
docker build -t gpu-worker .
docker run -p 8080:8080 gpu-worker
```

## Usage

### Starting the Service

```bash
# Default configuration
cargo run --release

# With custom configuration
HOST=0.0.0.0 PORT=3000 WORKERS=4 cargo run --release
```

### Environment Variables

- `HOST`: Server host (default: `0.0.0.0`)
- `PORT`: Server port (default: `8080`)
- `WORKERS`: Number of worker threads (default: CPU count)
- `RUST_LOG`: Log level (default: `info`)

## API Documentation

### Health Check

Check service health status.

```http
GET /health
GET /api/v1/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "gpu-worker",
  "version": "0.1.0",
  "features": ["mirror-gif"]
}
```

### Mirror GIF

Mirror a GIF image vertically.

```http
POST /mirror-gif
POST /api/v1/mirror-gif
Content-Type: multipart/form-data
```

**Request:**
- `file`: GIF file to process (multipart form field)

**Response:**
- Success: `200 OK` with mirrored GIF binary data
- Error: Appropriate error status with JSON error message

**Example:**
```bash
curl -X POST \
  -F "file=@input.gif" \
  http://localhost:8080/mirror-gif \
  -o output.gif
```

## Development

### Project Structure

```
gpu-worker/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── handlers.rs      # HTTP request handlers
│   └── error.rs         # Error types and handling
├── transformations/     # GPU transformation library
│   ├── src/
│   │   ├── lib.rs       # Library exports
│   │   ├── mirror.rs    # Mirror transformation
│   │   ├── blur.rs      # Blur transformation
│   │   ├── gpu.rs       # GPU processor base
│   │   └── error.rs     # Transformation errors
│   └── tests/           # Integration tests
├── tests/               # Service integration tests
├── benches/             # Performance benchmarks
└── examples/            # Usage examples
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --all

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run all tests
cargo test --all

# Run with coverage
cargo tarpaulin --out Html --all-features

# Run specific test
cargo test test_mirror_gif_success

# Run benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check dependencies
cargo audit
```

## CI/CD

The project uses GitHub Actions for continuous integration:

- **Testing**: Runs on every push and pull request
- **Code Coverage**: Uploads results to Codecov
- **Linting**: Enforces code style with rustfmt and clippy
- **Caching**: Speeds up builds with dependency caching

See `.github/workflows/ci.yml` for the complete pipeline configuration.

## Performance

The GPU acceleration provides significant performance improvements for image processing:

- Processes large GIFs (1920x1080) in milliseconds
- Handles concurrent requests efficiently
- Minimal CPU usage due to GPU offloading

## Troubleshooting

### GPU Not Found

If you encounter GPU initialization errors:

1. Ensure your GPU supports WebGPU
2. Update your graphics drivers
3. Try running with `WGPU_BACKEND=vulkan` or `WGPU_BACKEND=metal`

### Out of Memory

For large GIFs, you may need to increase the GPU memory limit or process in chunks.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Guidelines

- Write tests for new features
- Maintain code coverage above 80%
- Follow Rust naming conventions
- Document public APIs
- Update README for new features

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [wgpu](https://github.com/gfx-rs/wgpu) for WebGPU support
- Uses [actix-web](https://actix.rs/) for the web framework
- GIF processing with [gif](https://github.com/image-rs/image-gif) crate