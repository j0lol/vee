use crate::Model3d;
use glam::{DVec4, Mat4, Quat, Vec3};
use gltf::animation::util::ReadOutputs;
use std::collections::HashMap;
use std::path::Path;
use vee_models::model::{GenericModel3d, Vertex};
use vee_parse::NxCharInfo;
use vee_resources::{color::nx::linear::FAVORITE_COLOR, packing::Float16};
use wgpu::Color;

#[derive(Clone, Copy, Debug)]
struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    fn to_mat4(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

pub fn load_body<P: AsRef<Path>>(
    char_info: &NxCharInfo,
    path: P,
) -> Result<(Vec<Model3d>, Mat4), Box<dyn std::error::Error>> {
    let (gltf, buffers, _textures) = gltf::import(path)?;

    let mut local_transforms = get_local_transforms(&gltf);

    // Apply first-frame animation to avoid the default "t-pose".
    apply_animations(&gltf, &buffers, &mut local_transforms);

    // Move transforms relative to world root.
    let global_transforms = compute_global_transforms(&gltf, &local_transforms);

    // TODO: body scaling?

    // The head needs to be offset so it sits upon the body.
    // The issue with this is that the body is not always the same size, due to
    // body scaling. Therefore we have to pass it to the head "CharModel".
    let head_transform = find_labeled_transform("head", &gltf, &global_transforms)
        .expect("Should always be a `head` transform");

    let models = load_meshes(char_info, &gltf, &buffers, &global_transforms);

    Ok((models, head_transform))
}

fn get_local_transforms(gltf: &gltf::Document) -> HashMap<usize, Transform> {
    let mut local_transforms = HashMap::new();
    for node in gltf.nodes() {
        let (t, r, s) = node.transform().decomposed();
        local_transforms.insert(
            node.index(),
            Transform {
                translation: Vec3::from(t),
                rotation: Quat::from_array(r),
                scale: Vec3::from(s),
            },
        );
    }
    local_transforms
}

fn apply_animations(
    gltf: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    local_transforms: &mut HashMap<usize, Transform>,
) {
    if let Some(anim) = gltf
        .animations()
        .find(|a| a.name().is_some_and(|n| n.to_lowercase().contains("wait")))
    {
        for channel in anim.channels() {
            let target_node_index = channel.target().node().index();
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            // We don't care about later keyframes
            let _timestamps = reader.read_inputs().unwrap();
            let outputs = reader.read_outputs().unwrap(); // Values

            // We want the first keyframe, at least initially.
            // If we don't set an initial keyframe, it will default
            // to the initial "t-pose", which will look wrong.
            //
            // We implicitly take the first frame by calling `next`.
            if let Some(transform) = local_transforms.get_mut(&target_node_index) {
                match outputs {
                    ReadOutputs::Translations(mut iter) => {
                        if let Some(val) = iter.next() {
                            transform.translation = Vec3::from(val);
                        }
                    }
                    ReadOutputs::Rotations(iter) => {
                        if let Some(val) = iter.into_f32().next() {
                            transform.rotation = Quat::from_array(val);
                        }
                    }
                    ReadOutputs::Scales(mut iter) => {
                        if let Some(val) = iter.next() {
                            transform.scale = Vec3::from(val);
                        }
                    }
                    ReadOutputs::MorphTargetWeights(_) => {
                        // Weights not needed
                    }
                }
            }
        }
    }
}

fn compute_global_transforms(
    gltf: &gltf::Document,
    local_transforms: &HashMap<usize, Transform>,
) -> HashMap<usize, Mat4> {
    let mut global_transforms = HashMap::new();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            compute_node_transform(
                &node,
                Mat4::IDENTITY,
                local_transforms,
                &mut global_transforms,
            );
        }
    }
    global_transforms
}

fn compute_node_transform(
    node: &gltf::Node,
    parent_transform: Mat4,
    local_transforms: &HashMap<usize, Transform>,
    global_transforms: &mut HashMap<usize, Mat4>,
) {
    let local = local_transforms
        .get(&node.index())
        .map(|t| t.to_mat4())
        .unwrap_or(Mat4::IDENTITY);

    let global = parent_transform * local;
    global_transforms.insert(node.index(), global);

    for child in node.children() {
        compute_node_transform(&child, global, local_transforms, global_transforms);
    }
}

fn find_labeled_transform(
    label: &str,
    gltf: &gltf::Document,
    global_transforms: &HashMap<usize, Mat4>,
) -> Option<Mat4> {
    for node in gltf.nodes() {
        if node.name().is_some_and(|n| n.to_lowercase() == label)
            && let Some(t) = global_transforms.get(&node.index())
        {
            return Some(*t);
        }
    }
    None
}

fn load_meshes(
    char_info: &NxCharInfo,
    gltf: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    global_transforms: &HashMap<usize, Mat4>,
) -> Vec<Model3d> {
    let mut models = Vec::new();

    let favorite_color = FAVORITE_COLOR[char_info.favorite_color as usize];
    for node in gltf.nodes() {
        let color = match node.name().unwrap() {
            "body__mt_body" => favorite_color,
            "body__mt_pants" => [0.0, 1.0, 0.0], // todo find pants color
            _ => [1.0, 0.0, 1.0],                // evil magenta
        };

        if let Some(mesh) = node.mesh() {
            let node_global_transform = global_transforms
                .get(&node.index())
                .cloned()
                .unwrap_or(Mat4::IDENTITY);

            let skin = node.skin();
            let mut joint_matrices = Vec::new();

            if let Some(skin) = skin.as_ref() {
                let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

                // Transform bones to the correct position by:
                // 1. Mapping bones from model-space to local-space through IBMs
                let ibms: Vec<Mat4> = reader
                    .read_inverse_bind_matrices()
                    .map(|iter| iter.map(|m| Mat4::from_cols_array_2d(&m)).collect())
                    .unwrap_or_else(|| vec![Mat4::IDENTITY; skin.joints().count()]);

                for (i, joint) in skin.joints().enumerate() {
                    // 2. Mapping from local-space to global-space
                    let joint_global_transform = global_transforms
                        .get(&joint.index())
                        .cloned()
                        .unwrap_or(Mat4::IDENTITY);
                    let ibm = ibms.get(i).cloned().unwrap_or(Mat4::IDENTITY);

                    // 3. Apply Matrices in correct order
                    joint_matrices.push(joint_global_transform * ibm);
                }
            }

            for primitive in mesh.primitives() {
                models.push(primitive_to_model3d(
                    &primitive,
                    buffers,
                    &joint_matrices,
                    node_global_transform,
                    color,
                ));
            }
        }
    }
    models
}

fn primitive_to_model3d(
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
    joint_matrices: &[Mat4],
    node_global_transform: Mat4,
    color: [f32; 3],
) -> Model3d {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_default();

    let tex_coords: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|v| v.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|iter| iter.into_u32().collect())
        .unwrap_or_default();

    // Skinning attributes
    let joints: Vec<[u16; 4]> = reader
        .read_joints(0)
        .map(|iter| iter.into_u16().collect())
        .unwrap_or_else(|| vec![[0, 0, 0, 0]; positions.len()]);

    let weights: Vec<[f32; 4]> = reader
        .read_weights(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0, 0.0, 0.0]; positions.len()]);

    let mut final_vertices = Vec::new();

    for i in 0..positions.len() {
        let pos = Vec3::from(positions[i]);
        let norm = Vec3::from(normals[i]);

        let (pos, norm) = if !joint_matrices.is_empty() {
            // Associate bones to mesh (skinning)
            let j = joints[i];
            let w = weights[i];

            let mut skin_mat = Mat4::ZERO;
            for k in 0..4 {
                let joint_idx = j[k] as usize;
                if joint_idx < joint_matrices.len() {
                    skin_mat += joint_matrices[joint_idx] * w[k];
                }
            }

            // Apply skinning matrix
            (
                skin_mat.transform_point3(pos),
                skin_mat.transform_vector3(norm),
            )
        } else {
            // Rigid body transform, if there are no joints.
            (
                node_global_transform.transform_point3(pos),
                node_global_transform.transform_vector3(norm),
            )
        };

        final_vertices.push(Vertex {
            position: [
                Float16::from_f32(pos.x),
                Float16::from_f32(pos.y),
                Float16::from_f32(pos.z),
            ],
            _pad: 0,
            tex_coords: [
                Float16::from_f32(tex_coords[i][0]),
                Float16::from_f32(tex_coords[i][1]),
            ],
            normal: norm.to_array(),
        });
    }

    let color = Vec3::from_array(color).extend(1.0);

    GenericModel3d {
        vertices: final_vertices,
        indices,
        color,
        texture: None,
        position: Vec3::ZERO,
        scale: Vec3::splat(10.0),
    }
}
