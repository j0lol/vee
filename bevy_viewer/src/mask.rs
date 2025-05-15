use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use binrw::BinRead;
use std::{fs::File, io::BufReader};
use vfl::{
    charinfo::nx::NxCharInfo,
    mask::wgpu_render::{RenderContext, render_context_wgpu},
    shape_load::nx::{ResourceShape, SHAPE_MID_DAT, Shape},
    tex_load::nx::TEXTURE_MID_SRGB_DAT,
};

use crate::{
    CharDataRes,
    load::{self, load_mesh, setup_image},
};

fn draw_char_mask(
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands,
    render_layer: RenderLayers,
) {
    let mut tex_file = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());
    let mut tex_shape = BufReader::new(File::open(SHAPE_MID_DAT).unwrap());

    let mut char = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Jasmine.charinfo")).unwrap();
    let char = NxCharInfo::read(&mut char).unwrap();

    let image = futures::executor::block_on(render_context_wgpu(
        RenderContext::new(
            // &FaceParts::init(&char, 256.0),
            &char,
            (&mut tex_shape, &mut tex_file),
        )
        .unwrap(),
    ))
    .to_rgba8();

    image.save("test.png").unwrap();
    let image = setup_image(images, image.clone());

    commands.spawn((
        Sprite::from_image(image),
        render_layer,
        Transform::from_scale(vec3(1.0, -1.0, 1.0)),
    ));
}

pub fn setup_mask(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut res: ResMut<CharDataRes>,
) {
    res.0 = Some(load::get_res().unwrap());

    let size = Extent3d {
        width: 256,
        height: 256,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    draw_char_mask(&mut images, &mut commands, first_pass_layer.clone());

    commands.spawn((
        Camera2d,
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::NONE.into(),
            ..default()
        },
        first_pass_layer.clone(),
    ));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn({
        let meshes: &mut ResMut<Assets<Mesh>> = &mut meshes;
        let res: &ResourceShape = &res.0.unwrap();
        let shape = Shape::Mask;

        (
            Mesh3d(meshes.add(load_mesh(*res, shape, 1).unwrap())),
            MeshMaterial3d(material_handle),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.05)),
        )
    });
}

fn draw_char_glasses(
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands,
    render_layer: RenderLayers,
) {
    let mut tex_file = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());
    let mut tex_shape = BufReader::new(File::open(SHAPE_MID_DAT).unwrap());

    let mut char = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Jasmine.charinfo")).unwrap();
    let char = NxCharInfo::read(&mut char).unwrap();

    let image = futures::executor::block_on(render_context_wgpu(
        RenderContext::new_glasses(
            // &FaceParts::init_glasses(&char, 256.0),
            &char,
            (&mut tex_shape, &mut tex_file),
        )
        .unwrap(),
    ))
    .to_rgba8();

    image.save("testglasses.png").unwrap();
    let image = setup_image(images, image.clone());

    commands.spawn((
        Sprite::from_image(image),
        render_layer,
        Transform::from_scale(vec3(1.0, -1.0, 1.0)),
    ));
}

pub fn setup_glasses(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut res: ResMut<CharDataRes>,
) {
    res.0 = Some(load::get_res().unwrap());

    let size = Extent3d {
        width: 512,
        height: 256,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(2);

    draw_char_glasses(&mut images, &mut commands, first_pass_layer.clone());

    commands.spawn((
        Camera2d,
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::NONE.into(),
            ..default()
        },
        first_pass_layer.clone(),
    ));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn({
        let meshes: &mut ResMut<Assets<Mesh>> = &mut meshes;
        let res: &ResourceShape = &res.0.unwrap();
        let shape = Shape::Glasses;

        (
            Mesh3d(meshes.add(load_mesh(*res, shape, 0).unwrap())),
            MeshMaterial3d(material_handle),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.05)),
        )
    });
}
