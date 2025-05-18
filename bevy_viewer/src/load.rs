use crate::{CHAR_TRANSFORM, CharMesh};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use binrw::{BinRead, io::BufReader};
use image::{DynamicImage, RgbaImage};
use std::fs::File;
use vfl::{
    color::cafe::{FACELINE_COLOR, HAIR_COLOR},
    res::shape::nx::{GenericResourceShape, ResourceShape, SHAPE_MID_DAT, Shape, ShapeData},
};

fn shape_data_to_mesh(data: ShapeData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions.clone())
    .with_inserted_indices(Indices::U16(data.indices.clone()));

    if let Some(uvs) = data.uvs {
        mesh = mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    } else {
        println!("Missing UVs Probably an untextured shape eg HairNormal.");

        // if data.positions.len() == 4 {
        //     println!("Probably a Rect, adding Rect UVs.");

        //     let rect_uvs = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        //     let uvs: Vec<[f32; 2]> = rect_uvs
        //         .into_iter()
        //         .cycle()
        //         .take(data.positions.len())
        //         .collect();

        //     // let uvs: Vec<_> = data.positions.into_iter().map(|[x, y, z]| [x, y]).collect();
        //     mesh = mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        // } else {
        //     println!(
        //         "Unknown shape. positions: {} indices: {}",
        //         data.positions.len(),
        //         data.indices.len()
        //     );
        // }
    }

    if let Some(normals) = data.normals {
        mesh = mesh.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    } else {
        mesh = mesh.with_computed_normals();
    }

    mesh
}
pub fn load_mesh(res: ResourceShape, shape: Shape, hair_num: usize) -> Result<Mesh> {
    let GenericResourceShape::Element(mut shape) = res.fetch_shape(shape, hair_num).unwrap() else {
        panic!("wah")
    };

    #[cfg(target_family = "wasm")]
    let mesh = {
        use vee::shape_load::nx::SHAPE_MID_DAT_LOADED;
        let mut file = std::io::Cursor::new(SHAPE_MID_DAT_LOADED);
        shape.shape_data(&mut file).unwrap()
    };
    #[cfg(not(target_family = "wasm"))]
    let mesh = {
        let mut file = File::open(SHAPE_MID_DAT)?;
        shape.shape_data(&mut file).unwrap()
    };

    Ok(shape_data_to_mesh(mesh))
}

pub fn get_res() -> Result<ResourceShape> {
    #[cfg(target_family = "wasm")]
    {
        use vee::shape_load::nx::SHAPE_MID_DAT_LOADED;
        let mut bin = BufReader::new(std::io::Cursor::new(SHAPE_MID_DAT_LOADED));
        Ok(ResourceShape::read(&mut bin)?)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        let mut bin = BufReader::new(File::open(SHAPE_MID_DAT)?);
        Ok(ResourceShape::read(&mut bin)?)
    }
}

pub fn shape_bundle(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    res: &ResourceShape,
    hair_num: usize,
    color_num: usize,
    shape: Shape,
) -> impl Bundle {
    let [r, g, b, _a] = match shape {
        Shape::HairNormal => vfl::color::nx::linear::COMMON_COLOR[color_num],
        Shape::FaceLine | Shape::ForeheadNormal => FACELINE_COLOR[color_num],
        _ => [1.0, 0.0, 1.0, 1.0],
    };

    (
        Mesh3d(meshes.add(load_mesh(*res, shape, hair_num).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb(r, g, b))),
        CHAR_TRANSFORM,
        CharMesh,
    )
}

pub fn shape_tex_bundle(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    res: &ResourceShape,
    hair_num: usize,
    tex: Handle<Image>,
    shape: Shape,
) -> impl Bundle {
    (
        Mesh3d(meshes.add(load_mesh(*res, shape, hair_num).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial::from(tex))),
        CHAR_TRANSFORM,
        CharMesh,
    )
}

pub fn setup_image(images: &mut ResMut<Assets<Image>>, image: RgbaImage) -> Handle<Image> {
    let dynamic_image = DynamicImage::ImageRgba8(image);

    // Now add it to Bevy!
    images.add(Image::from_dynamic(
        dynamic_image,
        true,
        RenderAssetUsages::all(),
    ))
}
