use crate::{GuiData, MiiDataRes, MiiMesh};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use binrw::{BinRead, io::BufReader};
use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::fs::File;
use vee::{
    color::cafe::HAIR_COLOR,
    shape_load::nx::{GenericResourceShape, ResourceShape, SHAPE_MID_DAT, Shape, ShapeData},
};

fn shape_data_to_mesh(data: ShapeData) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
    .with_inserted_indices(Indices::U16(data.indices))
    .with_computed_normals();

    if let Some(uvs) = data.uvs {
        mesh = mesh.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
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
    let [r, g, b, _a] = HAIR_COLOR[color_num];

    (
        Mesh3d(meshes.add(load_mesh(*res, shape, hair_num).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb_from_array([r, g, b]))),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.05)),
        MiiMesh,
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
