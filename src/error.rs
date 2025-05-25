use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpuWorkerError {
    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WGPU device request error: {0}")]
    WgpuDevice(#[from] wgpu::RequestDeviceError),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("GIF decoding error: {0}")]
    GifDecode(#[from] gif::DecodingError),

    #[error("GIF encoding error: {0}")]
    GifEncode(#[from] gif::EncodingError),

    #[error("Transformation error: {0}")]
    Transformation(#[from] transformations::TransformationError),

    #[error("Multipart error: {0}")]
    Multipart(#[from] actix_multipart::MultipartError),
}

impl ResponseError for GpuWorkerError {
    fn error_response(&self) -> HttpResponse {
        let (mut status, error_type) = match self {
            Self::Gpu(_) | Self::WgpuDevice(_) | Self::Internal(_) | Self::Io(_) => {
                (HttpResponse::InternalServerError(), "internal_error")
            }
            Self::ImageProcessing(_) | Self::Image(_) | Self::GifDecode(_) | Self::GifEncode(_) => {
                (HttpResponse::UnprocessableEntity(), "processing_error")
            }
            Self::InvalidInput(_) | Self::Multipart(_) => {
                (HttpResponse::BadRequest(), "invalid_request")
            }
            Self::Transformation(_) => {
                (HttpResponse::InternalServerError(), "transformation_error")
            }
        };

        status.json(serde_json::json!({
            "error": error_type,
            "message": self.to_string()
        }))
    }
}

pub type Result<T> = std::result::Result<T, GpuWorkerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = GpuWorkerError::InvalidInput("test input".to_string());
        assert_eq!(error.to_string(), "Invalid input: test input");
    }

    #[test]
    fn test_error_response_status_codes() {
        let gpu_error = GpuWorkerError::Gpu("GPU failed".to_string());
        assert_eq!(gpu_error.error_response().status(), 500);

        let input_error = GpuWorkerError::InvalidInput("Bad input".to_string());
        assert_eq!(input_error.error_response().status(), 400);

        let processing_error = GpuWorkerError::ImageProcessing("Failed to process".to_string());
        assert_eq!(processing_error.error_response().status(), 422);
    }
}
