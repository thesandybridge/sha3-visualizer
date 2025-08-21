use cgmath::Point3;
use winit::{event::*, window::Window};
use std::sync::Arc;

use crate::{
    camera::{Camera, CameraController},
    renderer::Renderer,
    sha3::KeccakState,
    // ui::UI,
};

pub struct App {
    renderer: Renderer,
    camera: Camera,
    pub camera_controller: CameraController,
    // pub ui: UI,
    keccak_state: KeccakState,
    animation_time: f32,
    animation_speed: f32,
    is_playing: bool,
}

impl App {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let renderer = Renderer::new(window.clone(), size.width, size.height).await;
        
        let camera = Camera::new(
            Point3::new(0.0, 3.0, 8.0),    // Closer and slightly above center
            cgmath::Deg(-15.0),             // Look down slightly 
            cgmath::Deg(0.0),               // No roll
        );
        let camera_controller = CameraController::new(4.0, 0.4);
        
        // let ui = UI::new(&window, &renderer.device, renderer.surface_config.format);
        
        let mut keccak_state = KeccakState::new();
        // Set some test data so we can actually see something
        keccak_state.set_input("Hello SHA-3!");
        
        // Also manually set some bits for testing visualization
        for i in 0..5 {
            for j in 0..5 {
                for k in 0..10 {
                    if (i + j + k) % 3 == 0 {
                        // Manually set some bits to make them visible
                        let lane_idx = i + 5 * j;
                        if lane_idx < 25 {
                            keccak_state.state[lane_idx] |= 1u64 << k;
                        }
                    }
                }
            }
        }
        
        // Debug: print some state values
        println!("Keccak state initialized with {} lanes", keccak_state.state.len());
        for i in 0..5 {
            println!("Lane {}: {:#018x}", i, keccak_state.state[i]);
        }

        Self {
            renderer,
            camera,
            camera_controller,
            // ui,
            keccak_state,
            animation_time: 0.0,
            animation_speed: 1.0,
            is_playing: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(new_size);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event,
                ..
            } => {
                if let winit::keyboard::PhysicalKey::Code(key) = key_event.physical_key {
                    // Handle SHA-3 step controls
                    if key_event.state == winit::event::ElementState::Pressed {
                        match key {
                            winit::keyboard::KeyCode::Enter => {
                                // Step through SHA-3 algorithm
                                println!("ENTER pressed - stepping SHA-3");
                                self.keccak_state.step();
                                println!("Advanced to Round: {}, Step: {}", self.keccak_state.round, self.keccak_state.step);
                                return true;
                            }
                            winit::keyboard::KeyCode::KeyR => {
                                // Reset to initial state
                                println!("R pressed - resetting SHA-3");
                                self.keccak_state = KeccakState::new();
                                self.keccak_state.set_input("Hello SHA-3!");
                                println!("Reset SHA-3 state to Round: {}, Step: {}", self.keccak_state.round, self.keccak_state.step);
                                return true;
                            }
                            winit::keyboard::KeyCode::KeyP => {
                                // Toggle play/pause animation
                                self.is_playing = !self.is_playing;
                                println!("Animation {}", if self.is_playing { "playing" } else { "paused" });
                                return true;
                            }
                            _ => {}
                        }
                    }
                    
                    self.camera_controller.process_keyboard(key, key_event.state)
                } else {
                    false
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.camera_controller.process_mouse_click(*state);
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        
        if self.is_playing {
            self.animation_time += dt.as_secs_f32() * self.animation_speed;
            if self.animation_time >= 1.0 {
                self.animation_time = 0.0;
                self.keccak_state.step();
            }
        }
    }

    pub fn render(&mut self, _window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.renderer.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.renderer.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );

        // Update uniforms first
        self.renderer.uniforms.update_view_proj(&self.camera);
        self.renderer.queue.write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&[self.renderer.uniforms]));

        // Render the 3D scene
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.renderer.render_pipeline);
            render_pass.set_bind_group(0, &self.renderer.uniform_bind_group, &[]);

            // Render 5x5x64 Keccak matrix visualization
            let mut active_bits = 0;
            
            // First render grid outline (dark cubes) for visual structure
            for lane_x in 0..5 {
                for lane_z in 0..5 {
                    // Render a column outline every 8 bits to show structure
                    for bit_y in (0..64).step_by(8) {
                        let pos_x = (lane_x as f32 - 2.0) * 1.2;  // More spacing between lanes
                        let pos_y = bit_y as f32 * 0.15;          // More spacing between bits
                        let pos_z = (lane_z as f32 - 2.0) * 1.2;
                        
                        self.renderer.uniforms.update_model(pos_x, pos_y, pos_z);
                        self.renderer.queue.write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&[self.renderer.uniforms]));
                        self.renderer.cube_mesh.render_dark_at_position(&mut render_pass);
                    }
                }
            }
            
            // Then render active bits (bright cubes) on top
            for lane_x in 0..5 {
                for lane_z in 0..5 {
                    // Each lane has 64 bits stacked vertically
                    for bit_y in 0..64 {
                        let bit_is_set = self.keccak_state.get_bit(lane_x, bit_y, lane_z);
                        
                        if bit_is_set {
                            // Position: 5x5 base grid, with 64 bits stacked up
                            let pos_x = (lane_x as f32 - 2.0) * 1.2;  // 5 lanes across X - more spacing
                            let pos_y = bit_y as f32 * 0.15;          // 64 bits up Y - more spacing
                            let pos_z = (lane_z as f32 - 2.0) * 1.2;  // 5 lanes across Z - more spacing
                            
                            self.renderer.uniforms.update_model(pos_x, pos_y, pos_z);
                            self.renderer.queue.write_buffer(&self.renderer.uniform_buffer, 0, bytemuck::cast_slice(&[self.renderer.uniforms]));
                            self.renderer.cube_mesh.render_at_position(&mut render_pass);
                            
                            active_bits += 1;
                        }
                    }
                }
            }
        }

        // UI disabled for now due to lifetime issues

        self.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}