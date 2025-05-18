use crate::load::{load_mesh, setup_image, shape_bundle, shape_tex_bundle};
use crate::{CHAR_TRANSFORM, CHARINFO, CharDataRes, CharMesh, load};
use bevy::prelude::*;
use binrw::BinRead;
use std::{fs::File, io::BufReader};
use vfl::color::cafe::{FACELINE_COLOR, HAIR_COLOR};
use vfl::shape_load::nx::{ResourceShape, SHAPE_MID_DAT};
use vfl::{
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

    let mut char = File::open(CHARINFO).unwrap();

    let char = NxCharInfo::read(&mut char).unwrap();

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.faceline_type as usize,
        char.faceline_color as usize,
        dbg!(Shape::FaceLine),
    ));

    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        char.hair_type as usize,
        char.faceline_color as usize,
        dbg!(Shape::ForeheadNormal),
    ));

    // TODO: offset nose
    println!("Nose needs offsetting");

    let nose = false;
    if nose {
        let mut shape_file = BufReader::new(File::open(SHAPE_MID_DAT).unwrap());
        let res_shape = ResourceShape::read(&mut shape_file).unwrap();
        let nose_translate =
            res_shape.face_line_transform[char.faceline_type as usize].nose_translate;

        let shape_translation = vec3(
            nose_translate[0],
            5.0 + nose_translate[1] + -1.5 * f32::from(char.glass_y),
            2.0 + nose_translate[2],
        );
        let shape_scale = 0.15 * f32::from(char.glass_scale);

        commands.spawn({
            let materials: &mut ResMut<Assets<StandardMaterial>> = &mut materials;
            let meshes: &mut ResMut<Assets<Mesh>> = &mut meshes;
            let res: &ResourceShape = &res.0.unwrap();
            let hair_num = char.nose_type as usize;
            let color_num = char.faceline_color as usize;
            let shape = dbg!(Shape::Nose);
            let [r, g, b, _a] = match shape {
                Shape::HairNormal => HAIR_COLOR[color_num],
                Shape::FaceLine | Shape::ForeheadNormal | Shape::Nose => FACELINE_COLOR[color_num],
                _ => [0.4, 0.2, 0.6, 1.0],
            };

            (
                Mesh3d(meshes.add(load_mesh(*res, shape, hair_num).unwrap())),
                MeshMaterial3d(materials.add(Color::srgb(r, g, b))),
                Transform::from_translation(shape_translation * 0.05)
                    .with_scale(Vec3::splat(shape_scale) * 0.05),
                CharMesh,
            )
        });
    }

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
