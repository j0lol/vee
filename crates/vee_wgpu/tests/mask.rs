use common::setup_renderer_linear_color;
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
