use super::Vertex;
use crate::color::nx::ModulationIntent;
use image::DynamicImage;
use nalgebra::Matrix4;

type Color = [f32; 4];

#[derive(Debug)]
pub struct Model2d {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tex: DynamicImage,
    pub mvp_matrix: Matrix4<f32>,
    pub modulation: ModulationIntent,
    pub opaque: Option<Color>,
    pub label: Option<String>,
}
