#[cfg(test)]
mod inspections {
    use std::path::PathBuf;

    #[test]
    fn inspect_gltf() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../resources_here/miibodymiddle female test.glb");

        let (doc, _, _) = gltf::import(&path).unwrap();

        println!("Animations:");
        for anim in doc.animations() {
            println!("  Name: {:?}", anim.name());
            for channel in anim.channels() {
                println!(
                    "    Channel target: Node {:?}",
                    channel.target().node().name()
                );
                println!("    Path: {:?}", channel.target().property());
            }
        }

        println!("Skins:");
        for skin in doc.skins() {
            println!("  Name: {:?}", skin.name());
            println!("  Joints: {}", skin.joints().count());
        }

        println!("Nodes:");
        for node in doc.nodes() {
            println!(
                "  Node: {:?}, Mesh: {:?}, Skin: {:?}",
                node.name(),
                node.mesh().map(|m| m.name()),
                node.skin().map(|s| s.name())
            );
        }
    }
}
