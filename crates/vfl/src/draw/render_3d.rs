use glam::{Vec3, Vec4};

use super::Vertex;

#[derive(Default, Debug)]
pub struct GenericModel3d<Tex> {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub color: Vec4,
    pub texture: Option<Tex>,
    pub position: Vec3,
    pub scale: Vec3,
}
