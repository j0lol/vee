use crate::load::{load_mesh, setup_image, shape_bundle, shape_tex_bundle};
use crate::{CharDataRes, load};
use bevy::prelude::*;
use binrw::BinRead;
use std::{fs::File, io::BufReader};
use vee::{
    charinfo::nx::NxCharInfo,
    shape_load::nx::Shape,
    tex_load::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT},
};

pub fn setup_char(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut res: ResMut<CharDataRes>,
    mut images: ResMut<Assets<Image>>,
) {
    res.0 = Some(load::get_res().unwrap());

    let mut char = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../j0.charinfo")).unwrap();

    let char = NxCharInfo::read(&mut char).unwrap();

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.faceline_type as usize,
        4,
        Shape::FaceLine,
    ));

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.faceline_type as usize,
        4,
        Shape::ForeheadNormal,
    ));

    // Create and save a handle to the mesh.
    // Render the mesh with the custom texture, and add the marker.
    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.hair_type as usize,
        char.hair_color as usize,
        Shape::HairNormal,
    ));

    // get faceline transform
    // load_mesh(&res.0.unwrap(), Shape::FaceLineTransform, hair_num)

    let mut tex = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());
    let tex = ResourceTexture::read(&mut tex).unwrap();

    let glass_tex = tex.glass[2];
    let glass_tex = glass_tex
        .get_image(&mut BufReader::new(
            File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
        ))
        .unwrap()
        .unwrap();
    let glass_tex = setup_image(&mut images, glass_tex);

    // Create and save a handle to the mesh.
    // Render the mesh with the custom texture, and add the marker.
    commands.spawn(shape_tex_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        0,
        glass_tex,
        Shape::Glasses,
    ));
}
