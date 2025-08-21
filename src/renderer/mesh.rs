use wgpu::util::DeviceExt;
use crate::renderer::Vertex;

pub struct CubeMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    dark_vertex_buffer: wgpu::Buffer,
}

impl CubeMesh {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = &[
            // Bigger cubes for matrix bits - more visible
            Vertex { position: [-0.08, -0.08,  0.08], color: [1.0, 1.0, 1.0] }, // 0 - white
            Vertex { position: [ 0.08, -0.08,  0.08], color: [1.0, 1.0, 1.0] }, // 1
            Vertex { position: [ 0.08,  0.08,  0.08], color: [1.0, 1.0, 1.0] }, // 2
            Vertex { position: [-0.08,  0.08,  0.08], color: [1.0, 1.0, 1.0] }, // 3
            
            // Back face
            Vertex { position: [-0.08, -0.08, -0.08], color: [0.8, 0.8, 0.8] }, // 4 - light gray
            Vertex { position: [ 0.08, -0.08, -0.08], color: [0.8, 0.8, 0.8] }, // 5
            Vertex { position: [ 0.08,  0.08, -0.08], color: [0.8, 0.8, 0.8] }, // 6
            Vertex { position: [-0.08,  0.08, -0.08], color: [0.8, 0.8, 0.8] }, // 7
        ];

        let indices: &[u32] = &[
            // Front face
            0, 1, 2,  2, 3, 0,
            // Right face
            1, 5, 6,  6, 2, 1,
            // Back face
            7, 6, 5,  5, 4, 7,
            // Left face
            4, 0, 3,  3, 7, 4,
            // Bottom face
            4, 5, 1,  1, 0, 4,
            // Top face
            3, 2, 6,  6, 7, 3,
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        // Create dark/outline vertices for grid structure
        let dark_vertices = &[
            // Front face - very dark for outline
            Vertex { position: [-0.08, -0.08,  0.08], color: [0.1, 0.1, 0.1] }, // 0 - dark gray
            Vertex { position: [ 0.08, -0.08,  0.08], color: [0.1, 0.1, 0.1] }, // 1
            Vertex { position: [ 0.08,  0.08,  0.08], color: [0.1, 0.1, 0.1] }, // 2
            Vertex { position: [-0.08,  0.08,  0.08], color: [0.1, 0.1, 0.1] }, // 3
            
            // Back face
            Vertex { position: [-0.08, -0.08, -0.08], color: [0.05, 0.05, 0.05] }, // 4 - darker gray
            Vertex { position: [ 0.08, -0.08, -0.08], color: [0.05, 0.05, 0.05] }, // 5
            Vertex { position: [ 0.08,  0.08, -0.08], color: [0.05, 0.05, 0.05] }, // 6
            Vertex { position: [-0.08,  0.08, -0.08], color: [0.05, 0.05, 0.05] }, // 7
        ];

        let dark_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Dark Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(dark_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            dark_vertex_buffer,
        }
    }

    pub fn render_at_position<'rpass>(&self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    pub fn render_dark_at_position<'rpass>(&self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.dark_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}