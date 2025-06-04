use clap::{arg, Parser, Subcommand, ValueEnum};
use mesh_tools::compat::point3_new;
use mesh_tools::{GltfBuilder, Triangle};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::u8;
use vfl::parse::BinRead;
use vfl::res::packing::Float16;
use vfl::res::shape::{GenericResourceShape, ResourceShape, Shape, ShapeMesh};
use vfl::res::tex::{ResourceTexture, TextureElement};
// TODO: use real names
// https://github.com/ariankordi/ffl/blob/97eecdf3688f92c4c95cecf5d6ab3e84c0ee42c0/tools/FFLResource.py#L448
#[derive(Debug, Copy, Clone, ValueEnum)]
enum TextureType {
    Hat,
    Eye,
    Eyebrow,
    Beard,
    Wrinkle,
    Makeup,
    Glass,
    Mole,
    Mouth,
    Mustache,
    NoseLine,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, ValueEnum, IntoPrimitive)]
enum ArgShape {
    Beard = 0,
    FaceLine,
    Mask,
    HatNormal,
    HatCap,
    ForeheadNormal,
    ForeheadCap,
    HairNormal,
    HairCap,
    Glasses,
    Nose,
    NoseLine,
    HairTransform,
    FaceLineTransform,
}

/// Vee resource extractor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand, Debug, Clone)]
enum Subcommands {
    /// Read textures
    Texture {
        #[arg(short, long)]
        resource_file: PathBuf,
        #[arg(value_enum, short, long)]
        texture_type: TextureType,
        #[arg(short, long)]
        index: usize,
        output: PathBuf,
    },

    /// Show existant textures
    TextureExists {
        #[arg(short, long)]
        resource_file: PathBuf,
        #[arg(value_enum, short, long)]
        texture_type: TextureType,
    },

    /// Show existant textures
    ShapeExists {
        #[arg(short, long)]
        resource_file: PathBuf,
        #[arg(value_enum, short, long)]
        shape_type: ArgShape,
    },

    /// Read textures
    ShapeModel {
        #[arg(short, long)]
        resource_file: PathBuf,
        #[arg(value_enum, short, long)]
        shape_type: ArgShape,
        #[arg(short, long)]
        index: usize,
        output: PathBuf,
    },
}

// This is kind of `clap`-slop. I just need a quick debug tool.
fn main() {
    let args = Args::parse();

    match args.subcommands {
        Subcommands::Texture {
            resource_file,
            texture_type,
            index,
            output,
        } => {
            let res_tex = ResourceTexture::read(&mut BufReader::new(
                File::open(resource_file.clone()).unwrap(),
            ))
            .unwrap();

            let res_file = std::fs::read(resource_file).unwrap();

            // *YandereDev Voice* If only there was a better way...
            let texture_element = lookup_texture_type(texture_type, index, res_tex).unwrap();
            let el = texture_element.get_image(&res_file).unwrap().unwrap();
            el.save(output).unwrap();
        }
        Subcommands::TextureExists {
            resource_file,
            texture_type,
        } => {
            let res_tex = ResourceTexture::read(&mut BufReader::new(
                File::open(resource_file.clone()).unwrap(),
            ))
            .unwrap();
            let res_file = std::fs::read(resource_file).unwrap();

            let texture_type_length = match texture_type {
                TextureType::Hat => res_tex.hat.len(),
                TextureType::Eye => res_tex.eye.len(),
                TextureType::Eyebrow => res_tex.eyebrow.len(),
                TextureType::Beard => res_tex.beard.len(),
                TextureType::Wrinkle => res_tex.wrinkle.len(),
                TextureType::Makeup => res_tex.makeup.len(),
                TextureType::Glass => res_tex.glass.len(),
                TextureType::Mole => res_tex.mole.len(),
                TextureType::Mouth => res_tex.mouth.len(),
                TextureType::Mustache => res_tex.mustache.len(),
                TextureType::NoseLine => res_tex.noseline.len(),
            };
            let mut exists = Vec::with_capacity(texture_type_length);

            for index in 0..texture_type_length {
                let texture_element = lookup_texture_type(texture_type, index, res_tex).unwrap();
                if (texture_element.texture.width == 0 && texture_element.texture.height == 0)
                    || (texture_element.texture.width == 8 && texture_element.texture.height == 8)
                {
                    exists.push(false);
                    continue;
                }
                let texture = texture_element.get_image(&res_file).unwrap();

                let valid = texture.is_some();

                exists.push(valid);
            }

            println!(
                "{:#?}",
                exists
                    .iter()
                    .enumerate()
                    .filter_map(|(num, bool)| if *bool { Some(num) } else { None })
                    .collect::<Vec<_>>()
            )
        }
        Subcommands::ShapeExists {
            resource_file,
            shape_type,
        } => {
            let res_shape = ResourceShape::read(&mut BufReader::new(
                File::open(resource_file.clone()).unwrap(),
            ))
            .unwrap();
            let res_file = std::fs::read(resource_file).unwrap();

            let shape_type_len = match shape_type {
                ArgShape::Beard => res_shape.beard.len(),
                ArgShape::FaceLine => res_shape.face_line.len(),
                ArgShape::Mask => res_shape.mask.len(),
                ArgShape::HatNormal => res_shape.hat_normal.len(),
                ArgShape::HatCap => res_shape.hat_cap.len(),
                ArgShape::ForeheadNormal => res_shape.forehead_normal.len(),
                ArgShape::ForeheadCap => res_shape.forehead_cap.len(),
                ArgShape::HairNormal => res_shape.hair_normal.len(),
                ArgShape::HairCap => res_shape.hair_cap.len(),
                ArgShape::Glasses => res_shape.glasses.len(),
                ArgShape::Nose => res_shape.nose.len(),
                ArgShape::NoseLine => res_shape.nose_line.len(),
                ArgShape::HairTransform => res_shape.hair_transform.len(),
                ArgShape::FaceLineTransform => res_shape.face_line_transform.len(),
            };

            let mut exists = Vec::with_capacity(shape_type_len);

            for index in 0..shape_type_len {
                let shape = lookup_shape_type(shape_type, index, res_shape).unwrap();

                let GenericResourceShape::Element(mut shape) = shape else {
                    exists.push(false);
                    continue;
                };

                let mesh = shape.mesh(&res_file).unwrap();

                if mesh.positions.len() == 0 {
                    exists.push(false);

                    continue;
                }

                exists.push(true);
            }

            println!(
                "{:#?}",
                exists
                    .iter()
                    .enumerate()
                    .filter_map(|(num, bool)| if *bool { Some(num) } else { None })
                    .collect::<Vec<_>>()
            )
        }
        Subcommands::ShapeModel {
            resource_file,
            shape_type,
            index,
            output,
        } => {
            let res_shape = ResourceShape::read(&mut BufReader::new(
                File::open(resource_file.clone()).unwrap(),
            ))
            .unwrap();
            let res_file = std::fs::read(resource_file).unwrap();

            let shape = lookup_shape_type(shape_type, index, res_shape).unwrap();
            let GenericResourceShape::Element(mut shape) = shape else {
                panic!("Not a mesh!")
            };

            let mesh = shape.mesh(&res_file).unwrap();

            let mut builder = GltfBuilder::new();

            let ShapeMesh {
                positions,
                indices,
                normals: _,
                uvs: _,
                color_params: _,
            } = mesh;

            if positions.len() == 0 {
                println!("Empty model! Try again.");
                return;
            }
            let material = builder.create_basic_material(
                Some("Hair texture".to_string()),
                [0.118, 0.102, 0.094, 1.000],
            );

            let positions: Vec<_> = positions
                .into_iter()
                .map(|x| x.map(Float16::as_f32))
                .map(|[x, y, z, _]| point3_new(x, y, z))
                .collect();

            let indices: Vec<_> = indices
                .chunks_exact(3)
                .map(|s| <&[u16] as TryInto<[u16; 3]>>::try_into(s).unwrap()) // Thanks, Rust
                .map(|[x, y, z]| Triangle::new(x.into(), y.into(), z.into()))
                .collect();

            let mesh = builder.create_simple_mesh(
                Some("Mii Shape".to_string()),
                &positions,
                &indices,
                // normals.clone().map(|v| v.into_flattened()).as_deref(),
                // uvs.clone().map(|v| v.into_flattened()).as_deref(),
                None,
                None,
                Some(material),
            );

            let mii_shape_node = builder.add_node(
                Some("Mii Node".to_string()),
                Some(mesh),
                Some([0.0, 0.0, 0.0]),
                None,
                None,
            );

            builder.add_scene(Some("Mii Scene".to_string()), Some(vec![mii_shape_node]));

            builder.export_glb(&output.to_str().unwrap()).unwrap();
        }
    }
}

fn lookup_texture_type(
    texture_type: TextureType,
    index: usize,
    res_tex: ResourceTexture,
) -> Option<TextureElement> {
    match texture_type {
        TextureType::Hat => res_tex.hat.get(index).copied(),
        TextureType::Eye => res_tex.eye.get(index).copied(),
        TextureType::Eyebrow => res_tex.eyebrow.get(index).copied(),
        TextureType::Beard => res_tex.beard.get(index).copied(),
        TextureType::Wrinkle => res_tex.wrinkle.get(index).copied(),
        TextureType::Makeup => res_tex.makeup.get(index).copied(),
        TextureType::Glass => res_tex.glass.get(index).copied(),
        TextureType::Mole => res_tex.mole.get(index).copied(),
        TextureType::Mouth => res_tex.mouth.get(index).copied(),
        TextureType::Mustache => res_tex.mustache.get(index).copied(),
        TextureType::NoseLine => res_tex.noseline.get(index).copied(),
    }
}

fn lookup_shape_type(
    shape_type: ArgShape,
    index: usize,
    resource_shape: ResourceShape,
) -> Option<GenericResourceShape> {
    resource_shape.index_by_shape(
        Shape::try_from_primitive((shape_type).into()).unwrap(),
        index,
    )
}
