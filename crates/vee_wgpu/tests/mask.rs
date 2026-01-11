use approx::assert_relative_eq;
use common::{get_mask_data, setup_renderer_linear_color};
use vee_models::building::{MaskModels, mask_texture_meshes};
use vee_wgpu::ProgramState;

mod common;

#[test]
fn render_mask() {
    let mut e = setup_renderer_linear_color();

    let shapes = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data);

    for mut shape in shapes.all() {
        e.render
            .draw_model_2d(&mut shape, &e.texture.view, &mut e.encoder);
    }

    let image = e.render.output_texture(&e.texture, e.encoder);

    image
        .save(format!(
            "{}/test_data/outputs/mask.png",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
        ))
        .unwrap();
}

#[test]
fn render_mask_eyebrows() {
    let mut e = setup_renderer_linear_color();

    let meshes = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data);

    if meshes.right_brow.is_none() {
        return;
    }

    for mut item in meshes.all() {
        e.render
            .draw_model_2d(&mut item, &e.texture.view, &mut e.encoder);
    }

    let image = e.render.output_texture(&e.texture, e.encoder);

    image
        .save(format!(
            "{}/test_data/outputs/mask_brows.png",
            std::env::var("CARGO_WORKSPACE_DIR").unwrap(),
        ))
        .unwrap();
}

#[test]
fn mask_mtx() {
    let e = setup_renderer_linear_color();

    let test_mask = get_mask_data();

    let MaskModels {
        left_eye,
        right_eye,
        left_brow,
        right_brow,
        left_mustache,
        right_mustache,
        mouth,
        mole: _,
    } = mask_texture_meshes(&e.char, &e.texture_header, &e.texture_data);

    let comparisons = [
        (mouth, test_mask.mouth),
        (left_brow.unwrap(), test_mask.left_eyebrow),
        (right_brow.unwrap(), test_mask.right_eyebrow),
        (left_eye, test_mask.left_eye),
        (right_eye, test_mask.right_eye),
    ];

    for (mtx, mut test_mtx) in comparisons {
        {
            test_mtx.w_axis.y *= -1.0; // OpenGL -> WebGPU clip space (top-down to bottom-up)

            assert_relative_eq!(mtx.mvp_matrix, test_mtx);
        }
    }
}
