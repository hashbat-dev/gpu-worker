use wgpu::util::DeviceExt;
use crate::error::{TransformationError, Result};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] },
];

pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub struct GpuProcessor {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuProcessor {
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| TransformationError::GpuError("Failed to find an appropriate adapter".to_string()))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU Processor Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self { device, queue })
    }

    pub fn create_texture(&self, width: u32, height: u32, usage: wgpu::TextureUsages, label: &str) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage,
            view_formats: &[],
        })
    }

    pub fn create_buffer_init(&self, contents: &[u8], usage: wgpu::BufferUsages, label: &str) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage,
        })
    }

    pub fn create_vertex_buffer(&self) -> wgpu::Buffer {
        self.create_buffer_init(
            bytemuck::cast_slice(VERTICES),
            wgpu::BufferUsages::VERTEX,
            "Vertex Buffer",
        )
    }

    pub fn create_index_buffer(&self) -> wgpu::Buffer {
        self.create_buffer_init(
            bytemuck::cast_slice(INDICES),
            wgpu::BufferUsages::INDEX,
            "Index Buffer",
        )
    }

    pub fn create_sampler(&self) -> wgpu::Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    pub async fn read_buffer(&self, buffer: &wgpu::Buffer, _size: u64) -> Result<Vec<u8>> {
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        
        self.device.poll(wgpu::Maintain::Wait);
        
        rx.await
            .map_err(|_| TransformationError::BufferError("Failed to receive buffer mapping result".to_string()))?
            .map_err(|e| TransformationError::BufferError(format!("Failed to map buffer: {:?}", e)))?;
        
        let data = buffer_slice.get_mapped_range().to_vec();
        let _ = buffer_slice;
        buffer.unmap();
        
        Ok(data)
    }

    pub fn calculate_aligned_bytes_per_row(&self, width: u32) -> u32 {
        let unpadded_bytes_per_row = 4 * width;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        (unpadded_bytes_per_row + align - 1) / align * align
    }

    pub fn remove_padding(&self, data: &[u8], width: u32, height: u32, padded_bytes_per_row: u32) -> Vec<u8> {
        let unpadded_bytes_per_row = 4 * width;
        
        if padded_bytes_per_row == unpadded_bytes_per_row {
            data.to_vec()
        } else {
            let mut result = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
            for row in 0..height {
                let row_start = (row * padded_bytes_per_row) as usize;
                let row_end = row_start + unpadded_bytes_per_row as usize;
                result.extend_from_slice(&data[row_start..row_end]);
            }
            result
        }
    }
}