use nalgebra::{Matrix3x4, Matrix4, Rotation3, Scale2, Vector2, Vector3, matrix, stack};

pub struct RawMaskPartsDesc {
    scale: Vector2<f32>,
    pos: Vector2<f32>,
    rotation_rads: f32,
}

impl RawMaskPartsDesc {}

type Mat34 = Matrix3x4<f32>;

pub fn transformation_matrix(
    scale: Vector2<f32>,
    translation: Vector2<f32>,
    rotation_rads: f32,
) -> Matrix4<f32> {
    let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), rotation_rads);
    let rotation = rotation.matrix();

    let scale = Scale2::new(scale.x, scale.y).to_homogeneous();

    let rot_and_scale = rotation * scale;

    let translation = matrix![translation.x; translation.y; 1.0];

    let one = matrix![1.0];
    let mtx = stack![rot_and_scale, translation; 0, one];

    mtx
}

/// CalcMVMatrix
pub fn calc_mv_matrix(p_mv_matrix: &mut Mat34, p_desc: &RawMaskPartsDesc) {
    // Mat34 m;

    // MAT34Scale(pMVMatrix, pDesc->scale.x, pDesc->scale.y, 1.0f);
    p_mv_matrix.row_mut(0).scale_mut(p_desc.scale.x);
    p_mv_matrix.row_mut(1).scale_mut(p_desc.scale.y);

    let transform_matrix = transformation_matrix(
        Vector2::<f32>::new(0.889_614_64, 0.927_667_5),
        p_desc.pos,
        p_desc.rotation_rads,
    );

    *p_mv_matrix *= transform_matrix;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, f32::consts::PI};

    type R = Result<(), Box<dyn Error>>;

    #[test]
    fn scale_test() {
        let mut mtx = Matrix3x4::identity();

        calc_mv_matrix(
            &mut mtx,
            &RawMaskPartsDesc {
                scale: Vector2::<f32>::new(1.0, 1.0),
                pos: Vector2::zeros(),
                rotation_rads: PI,
            },
        );
    }
}
