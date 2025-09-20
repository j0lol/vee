use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod app;
pub mod camera;
pub mod state;

const DARK_REBECCA_PURPLE: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

const HIGHLIGHT_BLUE: wgpu::Color = wgpu::Color {
    r: 0.7,
    g: 0.7,
    b: 1.0,
    a: 0.5,
};

const FACES: [&str; 1] = [
    // "chris.charinfo",
    // "aspect.charinfo",
    // "Jasmine.charinfo",
    // "Aiueome.charinfo",
    "Bro Mole High.charinfo",
    // "alien fcln.charinfo",
    // "testguy.charinfo",
    // "j0.charinfo",
    // "charline.charinfo",
    // "soyun.charinfo",
];

/// There's not much here. Look in `State::new`/`render` for real logic
fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
