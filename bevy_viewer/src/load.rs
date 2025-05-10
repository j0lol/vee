use crate::MiiMesh;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use binrw::{BinRead, io::BufReader};
use std::fs::File;
use vee::{
    color::cafe::HAIR_COLOR,
    shape_load::nx::{GenericResourceShape, ResourceShape, Shape, ShapeData},
};

fn shape_data_to_mesh(data: ShapeData) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
    .with_inserted_indices(Indices::U16(data.indices))
    .with_computed_normals()
}
pub fn load_mesh(res: ResourceShape, shape: Shape, hair_num: usize) -> Result<Mesh> {
    let GenericResourceShape::Element(mut shape) = res.fetch_shape(shape, hair_num).unwrap() else {
        panic!("wah")
    };
    let mut file = File::open("ShapeMid.dat")?;
    let mesh = shape.shape_data(&mut file).unwrap();

    Ok(shape_data_to_mesh(mesh))
}

pub fn get_res() -> Result<ResourceShape> {
    let mut bin = BufReader::new(File::open("ShapeMid.dat")?);
    Ok(ResourceShape::read(&mut bin)?)
}

pub fn shape_bundle(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    res: &ResourceShape,
    hair_num: usize,
    color_num: usize,
) -> impl Bundle {
    let [r, g, b, _a] = HAIR_COLOR[color_num];

    (
        Mesh3d(meshes.add(load_mesh(*res, Shape::HairNormal, hair_num).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb_from_array([r, g, b]))),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.05)),
        MiiMesh,
    )
}
