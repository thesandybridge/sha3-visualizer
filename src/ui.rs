use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use winit::window::Window;

use crate::sha3::KeccakState;

pub struct UI {
    egui_ctx: egui::Context,
    egui_state: State,
    egui_renderer: egui_wgpu::Renderer,
    input_text: String,
}

impl UI {
    pub fn new(window: &Window, device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui_ctx.viewport_id(), &window, None, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1, false);

        Self {
            egui_ctx,
            egui_state,
            egui_renderer,
            input_text: "Hello, SHA-3!".to_string(),
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        keccak_state: &mut KeccakState,
        animation_speed: &mut f32,
        is_playing: &mut bool,
    ) {
        let raw_input = self.egui_state.take_egui_input(&window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            egui::Window::new("SHA-3 Controls")
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Input");
                    
                    ui.horizontal(|ui| {
                        ui.label("Text to hash:");
                        if ui.text_edit_singleline(&mut self.input_text).changed() {
                            keccak_state.set_input(&self.input_text);
                        }
                    });

                    ui.separator();

                    ui.heading("Animation");
                    
                    ui.horizontal(|ui| {
                        if ui.button(if *is_playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                            *is_playing = !*is_playing;
                        }
                        
                        if ui.button("⏹ Reset").clicked() {
                            keccak_state.set_input(&self.input_text);
                            *is_playing = false;
                        }
                        
                        if ui.button("⏭ Step").clicked() {
                            keccak_state.step();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.add(egui::Slider::new(animation_speed, 0.1..=5.0).text("x"));
                    });

                    ui.separator();

                    ui.heading("Current State");
                    ui.label(format!("Current step: {}", keccak_state.get_current_step_name()));
                    
                    ui.separator();

                    ui.heading("Hash Output");
                    ui.horizontal(|ui| {
                        let output = keccak_state.get_output_hex();
                        ui.text_edit_singleline(&mut output.clone());
                        if ui.button("📋").clicked() {
                            ui.output_mut(|o| o.copied_text = output);
                        }
                    });

                    ui.separator();

                    ui.heading("History");
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, step) in keccak_state.get_history().iter().enumerate().rev() {
                            ui.collapsing(format!("{}. Round {} - {}", i + 1, step.round, step.step_name), |ui| {
                                ui.label(&step.description);
                            });
                        }
                    });
                });

            egui::Window::new("Camera Controls")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Controls");
                    ui.label("WASD - Move camera");
                    ui.label("Mouse wheel - Zoom");
                    ui.label("Space/Shift - Up/Down");
                    ui.label("Mouse drag - Look around");
                });
        });

        self.egui_state.handle_platform_output(&window, full_output.platform_output);

        let tris = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, &image_delta);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        self.egui_renderer.update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.egui_renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x)
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(&window, event);
    }
}