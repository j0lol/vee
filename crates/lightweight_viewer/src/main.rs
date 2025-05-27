use app::App;
use glam::{vec4, Vec4};
use winit::event_loop::{ControlFlow, EventLoop};

pub mod app;
pub mod char_draw;
pub mod char_model;
pub mod state;

const DARK_REBECCA_PURPLE: wgpu::Color = wgpu::Color {
    r: 0.2,
    g: 0.1,
    b: 0.3,
    a: 1.0,
};

pub const fn wgpu_color_to_vec4(color: wgpu::Color) -> Vec4 {
    vec4(
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    )
}

const FACES: [&str; 4] = [
    // "testguy.charinfo",
    "j0.charinfo",
    "charline.charinfo",
    "Jasmine.charinfo",
    "soyun.charinfo",
];

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

mod camera {
    use glam::{Mat4, Vec3};

    pub struct Camera {
        pub eye: Vec3,
        pub target: Vec3,
        pub up: Vec3,
        pub aspect: f32,
        pub fov_y_radians: f32,
        pub znear: f32,
        pub zfar: f32,
    }

    impl Camera {
        pub fn build_view_projection_matrix(&self) -> Mat4 {
            let view = Mat4::look_at_rh(self.eye, self.target, self.up);
            let proj = Mat4::perspective_rh(self.fov_y_radians, self.aspect, self.znear, self.zfar);
            proj * view
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct CameraUniform {
        view_proj: [[f32; 4]; 4],
    }

    impl CameraUniform {
        pub fn new() -> Self {
            Self {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }
        }

        pub fn update_view_proj(&mut self, camera: &Camera) {
            self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
        }
    }
}
