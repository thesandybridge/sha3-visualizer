use cgmath::*;
use std::time::Duration;
use winit::event::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            aspect: 1.0,
            fovy: Rad(std::f32::consts::FRAC_PI_4),
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Matrix4::look_at_rh(
            self.position,
            self.position
                + Vector3::new(
                    cos_pitch * cos_yaw,
                    sin_pitch,
                    cos_pitch * sin_yaw,
                ),
            Vector3::unit_y(),
        )
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = self.calc_matrix();
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    pub is_mouse_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            is_mouse_pressed: false,
        }
    }

    pub fn process_keyboard(&mut self, key: winit::keyboard::KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            winit::keyboard::KeyCode::KeyW | winit::keyboard::KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyS | winit::keyboard::KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            winit::keyboard::KeyCode::KeyA | winit::keyboard::KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            winit::keyboard::KeyCode::KeyD | winit::keyboard::KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            winit::keyboard::KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            winit::keyboard::KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse_click(&mut self, state: ElementState) {
        self.is_mouse_pressed = state == ElementState::Pressed;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition { y: scroll, .. }) => {
                *scroll as f32
            }
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move up/down
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(std::f32::consts::FRAC_PI_2) {
            camera.pitch = -Rad(std::f32::consts::FRAC_PI_2);
        } else if camera.pitch > Rad(std::f32::consts::FRAC_PI_2) {
            camera.pitch = Rad(std::f32::consts::FRAC_PI_2);
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }
}