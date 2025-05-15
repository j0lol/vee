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

    let mut char = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../Jasmine.charinfo")).unwrap();

    let char = NxCharInfo::read(&mut char).unwrap();

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.faceline_type as usize,
        4,
        dbg!(Shape::FaceLine),
    ));

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.faceline_type as usize,
        4,
        dbg!(Shape::ForeheadNormal),
    ));

    // Create and save a handle to the mesh.
    // Render the mesh with the custom texture, and add the marker.
    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.hair_type as usize,
        char.hair_color as usize,
        dbg!(Shape::HairNormal),
    ));
}
