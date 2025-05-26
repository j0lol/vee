use glam::Mat4;
use super::Vertex;
use crate::color::nx::ModulationIntent;
use image::DynamicImage;

type Color = [f32; 4];

#[derive(Debug)]
pub struct Model2d {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tex: DynamicImage,
    pub mvp_matrix: Mat4,
    pub modulation: ModulationIntent,
    pub opaque: Option<Color>,
    pub label: Option<String>,
}
