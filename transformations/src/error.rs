use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformationError {
    #[error("GPU error: {0}")]
    GpuError(String),
    
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("Buffer error: {0}")]
    BufferError(String),
}

pub type Result<T> = std::result::Result<T, TransformationError>;

impl From<wgpu::RequestDeviceError> for TransformationError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        TransformationError::GpuError(format!("Failed to request device: {}", err))
    }
}

impl From<wgpu::BufferAsyncError> for TransformationError {
    fn from(err: wgpu::BufferAsyncError) -> Self {
        TransformationError::GpuError(format!("Buffer async error: {}", err))
    }
}