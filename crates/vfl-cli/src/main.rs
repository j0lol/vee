use clap::{arg, Parser, Subcommand, ValueEnum};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use vfl::parse::BinRead;
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
