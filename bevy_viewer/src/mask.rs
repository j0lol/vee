use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use bevy_image_export::{ImageExport, ImageExportSettings, ImageExportSource};
use binrw::BinRead;
use std::{fs::File, io::BufReader};
use vee::{
    charinfo::nx::NxCharInfo,
    mask::{FacePart, FaceParts},
    shape_load::nx::{ResourceShape, Shape},
    tex_load::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT, TextureElement},
};

use crate::{
    CharDataRes,
    load::{self, load_mesh, setup_image},
};

fn draw_mask_part(
    part: &FacePart,
    tex: &TextureElement,
    // file: &mut BufReader<File>,
    images: &mut ResMut<Assets<Image>>,
) -> impl Bundle {
    let tex = tex
        .get_image(&mut BufReader::new(
            File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
        ))
        .unwrap()
        .unwrap();

    let img = setup_image(images, tex.clone());

    // TODO: Made up offsets. Reference RFL and FFL for proper offsets.
    let origin_offset = match part.origin {
        vee::mask::ImageOrigin::Center => vec2(0.0, -1.0),
        vee::mask::ImageOrigin::Left => vec2(-1.5, 0.0),
        vee::mask::ImageOrigin::Right => vec2(1.5, 0.0),
    } * 50.0;

    let flip = if part.origin == vee::mask::ImageOrigin::Right {
        vec2(-1.0, 1.0)
    } else {
        Vec2::ONE
    };

    // textured quad
    (
        Sprite::from_image(img),
        // Mesh3d(quad_handle.clone()),
        // MeshMaterial3d(material_handle),
        Transform::from_xyz(
            (part.x / 256.0) + origin_offset.x,
            (part.y / 256.0) + origin_offset.y,
            1.0,
        )
        .with_scale(
            (vec2(
                tex.width() as f32 / part.width / 4.0,
                tex.height() as f32 / part.height / 4.0,
            ) * flip)
                .extend(1.0),
        ),
    )
}
fn draw_char_mask(
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands,
    render_layer: RenderLayers,
) {
    let mut char = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../j0.charinfo")).unwrap();
    let char = NxCharInfo::read(&mut char).unwrap();

    let mask_info = FaceParts::init(&char, 256.0);

    let mut tex = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());
    let tex = ResourceTexture::read(&mut tex).unwrap();

    commands.spawn((
        draw_mask_part(&mask_info.eye[0], &tex.eye[char.eye_type as usize], images),
        render_layer.clone(),
    ));

    commands.spawn((
        draw_mask_part(&mask_info.eye[1], &tex.eye[char.eye_type as usize], images),
        render_layer.clone(),
    ));

    commands.spawn((
        draw_mask_part(
            &mask_info.mouth,
            &tex.mouth[char.mouth_type as usize],
            images,
        ),
        render_layer.clone(),
    ));

    commands.spawn((
        draw_mask_part(
            &mask_info.eyebrow[0],
            &tex.eyebrow[char.eyebrow_type as usize],
            images,
        ),
        render_layer.clone(),
    ));

    commands.spawn((
        draw_mask_part(
            &mask_info.eyebrow[1],
            &tex.eyebrow[char.eyebrow_type as usize],
            images,
        ),
        render_layer.clone(),
    ));
}

pub fn setup_mask(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut res: ResMut<CharDataRes>,
    // mut export_sources: ResMut<Assets<ImageExportSource>>,
) {
    res.0 = Some(load::get_res().unwrap());

    let size = Extent3d {
        width: 512,
        height: 512,
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

    // commands.spawn((
    //     ImageExport(export_sources.add(image_handle.clone())),
    //     ImageExportSettings {
    //         // Frames will be saved to "./out/[#####].png".
    //         output_dir: "out".into(),
    //         // Choose "exr" for HDR renders.
    //         extension: "png".into(),
    //     },
    // ));

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    draw_char_mask(&mut images, &mut commands, first_pass_layer.clone());

    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        first_pass_layer.clone(),
    ));

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
