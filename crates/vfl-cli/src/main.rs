use clap::{arg, Parser, Subcommand, ValueEnum};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use vfl::charinfo::nx::BinRead;

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
            let res_tex = vfl::res::tex::nx::ResourceTexture::read(&mut BufReader::new(
                File::open(resource_file.clone()).unwrap(),
            ))
            .unwrap();

            let res_file = std::fs::read(resource_file).unwrap();

            // *YandereDev Voice* If only there was a better way...
            let texture_element = match texture_type {
                TextureType::Hat => res_tex.hat[index],
                TextureType::Eye => res_tex.eye[index],
                TextureType::Eyebrow => res_tex.eyebrow[index],
                TextureType::Beard => res_tex.beard[index],
                TextureType::Wrinkle => res_tex.wrinkle[index],
                TextureType::Makeup => res_tex.makeup[index],
                TextureType::Glass => res_tex.glass[index],
                TextureType::Mole => res_tex.mole[index],
                TextureType::Mouth => res_tex.mouth[index],
                TextureType::Mustache => res_tex.mustache[index],
                TextureType::NoseLine => res_tex.noseline[index],
            };
            let el = texture_element.get_image(&res_file).unwrap().unwrap();
            el.save(output).unwrap();
        }
    }
}
