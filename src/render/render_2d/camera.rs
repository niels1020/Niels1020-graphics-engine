use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferUsages, Device, Queue, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::common::CAMERA_BINDING;

pub struct Camera2D {
    pub(crate) layout: Option<BindGroupLayout>,
    pub(crate) bind: Option<BindGroup>,
    buffer: Option<Buffer>,
    pub data: Camera2DData,
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera2DData {
    pub position: [f32; 2],
    pub render_resolution: [f32; 2],
}

impl Camera2D {
    pub fn new(render_resolution: [f32; 2]) -> Self {
        Self {
            layout: None,
            buffer: None,
            bind: None,
            data: Camera2DData {
                position: [0.0, 0.0],
                render_resolution,
            },
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        if self.layout.is_none() {
            self.layout = Some(create_bind_group_layout(device))
        }

        if self.buffer.is_none() {
            self.buffer = Some(create_buffer(device, self.data))
        }

        if self.bind.is_none() {
            self.bind = Some(create_bind_group(
                device,
                self.layout.as_ref().unwrap(),
                self.buffer.as_ref().unwrap(),
            ))
        }

        queue.write_buffer(
            self.buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[self.data]),
        );
    }
}

//always binding 0
fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("a Camera2D bindgroup layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_buffer(device: &Device, data: Camera2DData) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some("a Camera2D buffer"),
        contents: bytemuck::cast_slice(&[data]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

fn create_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("a Camera2D bindgroup"),
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}
