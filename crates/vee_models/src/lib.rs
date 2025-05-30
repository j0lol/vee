//! Library for turning parsed models into real vertex and texture data, ready for rendering.
pub mod building;
pub mod model;
pub mod positioning;

pub use model::GenericModel3d;
pub use model::Model2d;

const TEX_SCALE_X: f32 = 0.889_614_64;
const TEX_SCALE_Y: f32 = 0.927_667_5;
