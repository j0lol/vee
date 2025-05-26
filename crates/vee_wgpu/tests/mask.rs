use std::{path::PathBuf, str::FromStr};

use approx::assert_relative_eq;
use common::{ffl_runner::FFlRunner, get_mask_data, setup_renderer_linear_color};
use vee_wgpu::ProgramState;
use vfl::draw::mesh_building::mask_texture_meshes;

mod common;

#[test]
fn render_mask() {
    let mut e = setup_renderer_linear_color();

    let shapes = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data);

    for mut shape in shapes {
        e.render
            .draw_model_2d(&mut shape, &e.texture.view, &mut e.encoder);
    }

    let image = e.render.output_texture(&e.texture, e.encoder);

    image
        .save(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/test_data/outputs/mask.png",
        ))
        .unwrap();
}

#[test]
fn render_mask_eyebrows() {
    let mut e = setup_renderer_linear_color();

    let [ref _el, ref _er, ref mut ebl, ref mut ebr, ref _mouth] =
        mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data)[..]
    else {
        return;
    };

    for shape in [ebl, ebr] {
        e.render
            .draw_model_2d(shape, &e.texture.view, &mut e.encoder);
    }

    let image = e.render.output_texture(&e.texture, e.encoder);

    image
        .save(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/test_data/outputs/mask_brows.png",
        ))
        .unwrap();
}

#[test]
fn mask_mtx() {
    let e = setup_renderer_linear_color();

    let test_mask = get_mask_data();

    let [
        ref left_eye,
        ref right_eye,
        ref left_brow,
        ref right_brow,
        ref mouth,
    ] = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data)[..]
    else {
        panic!()
    };

    let comparisons = [
        (mouth.mvp_matrix, test_mask.mouth),
        (left_brow.mvp_matrix, test_mask.left_eyebrow),
        (right_brow.mvp_matrix, test_mask.right_eyebrow),
        (left_eye.mvp_matrix, test_mask.left_eye),
        (right_eye.mvp_matrix, test_mask.right_eye),
    ];

    for (mut mtx, mut test_mtx) in comparisons {
        test_mtx.w_axis.y *= -1.0; // OpenGL -> WebGPU clip space (top-down to bottom-up)

        assert_relative_eq!(mtx, test_mtx);
    }
}

// #[test]
// fn mask_mtx() {
//     let e = setup_renderer_linear_color();
//     let mut ffl = FFlRunner {
//         dir: PathBuf::from_str(concat!(env!("CARGO_WORKSPACE_DIR"), "../FFL-Testing/")).unwrap(),
//     };

//     // This takes about two seconds.
//     ffl.run_ffl_testing().unwrap();

//     let [
//         ref _left_eye,
//         ref _right_eye,
//         ref left_brow,
//         ref right_brow,
//         ref mouth,
//     ] = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data)[..]
//     else {
//         panic!()
//     };

//     let comparisons = [(mouth, "mouthmtx.txt"), (left_brow, "brow0mtx.txt"), (right_brow, "brow1mtx.txt")];

//     for (model, file) in comparisons {
//         let mut mtx = model.mvp_matrix;
//         *mtx.get_mut((1, 1)).unwrap() *= -1.0; // We need to flip the y axis because opengl blablablablalbalbal

//         let ffl_mtx = ffl.get_resultant_mtx44(file).unwrap();

//         assert_relative_eq!(mtx, ffl_mtx);
//     }
// }
