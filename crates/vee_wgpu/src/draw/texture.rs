use crate::ProgramState;
use image::DynamicImage;
use vee_models::building::mask_texture_meshes;
use vee_models::model::DrawableTexture;
use vee_parse::NxCharInfo;
use vee_resources::color;
use vee_resources::color::nx::{ColorModulated, ModulationIntent, modulate};
use vee_resources::tex::TextureElement;
use wgpu::{CommandEncoder, TextureView};

pub(crate) fn draw_noseline(
    st: &mut dyn ProgramState,
    char_info: &NxCharInfo,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let res_texture = &st.texture_header();
    let file_texture = st.texture_data();

    let noseline_num = usize::from(char_info.nose_type);

    let tex: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = res_texture.noseline[noseline_num]
        .get_image(file_texture.as_ref())
        .unwrap()
        .unwrap();
    let tex = DynamicImage::ImageRgba8(tex);

    st.draw_texture(
        DrawableTexture {
            rendered_texture: tex,
            modulation: modulate(ColorModulated::NoseLineShape, char_info),
            opaque: None,
        },
        texture_view,
        encoder,
    );
}

pub(crate) fn draw_mask(
    st: &mut dyn ProgramState,
    char_info: &NxCharInfo,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let res_texture = &st.texture_header();
    let file_texture = &st.texture_data();

    let shapes = mask_texture_meshes(char_info, res_texture, file_texture);

    for mut shape in shapes.all() {
        st.draw_model_2d(&mut shape, texture_view, encoder);
    }
}

/// Looks up a TextureElement, and returns the texture with any modulation that needs to be done.
/// Returns an `Option<T>` because the texture could not exist (e.g. `CharInfo` w/o `Beard`)
fn load_faceline_texture(
    st: &mut (impl ProgramState + ?Sized),
    char_info: &NxCharInfo,
    texture_element: TextureElement,
    modulated: ColorModulated,
) -> Option<(DynamicImage, ModulationIntent)> {
    texture_element
        .get_image(&st.texture_data())
        .unwrap()
        .map(|tex| {
            (
                DynamicImage::ImageRgba8(tex),
                modulate(modulated, char_info),
            )
        })
}

// Load faceline textures in order [wrinkle, makeup, beard], and removes any that don't exist
fn get_faceline_textures(
    st: &mut (impl ProgramState + ?Sized),
    char_info: &NxCharInfo,
) -> Vec<(DynamicImage, ModulationIntent)> {
    let texture_header = st.texture_header();
    vec![
        {
            if char_info.faceline_wrinkle != 0 {
                load_faceline_texture(
                    st,
                    char_info,
                    texture_header.wrinkle[char_info.faceline_wrinkle as usize],
                    ColorModulated::FacelineWrinkle,
                )
            } else {
                None
            }
        },
        {
            if char_info.faceline_make != 0 {
                load_faceline_texture(
                    st,
                    char_info,
                    texture_header.makeup[char_info.faceline_make as usize],
                    ColorModulated::FacelineMakeup,
                )
            } else {
                None
            }
        },
        {
            if char_info.beard_type >= 4 {
                load_faceline_texture(
                    st,
                    char_info,
                    texture_header.beard[usize::from(char_info.beard_type - 4)],
                    ColorModulated::FacelineBeard,
                )
            } else {
                None
            }
        },
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub(crate) fn draw_faceline(
    st: &mut dyn ProgramState,
    char_info: &NxCharInfo,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let textures = get_faceline_textures(st, char_info);

    for (i, (rendered_texture, modulation)) in textures.into_iter().enumerate() {
        // Check if we are the first to be rendered out, then add an opaque background.
        // We don't want an opaque redraw happening over our other faceline textures.
        let opaque = (i == 0)
            .then_some(color::nx::linear::FACELINE_COLOR[usize::from(char_info.faceline_color.0)]);

        st.draw_texture(
            DrawableTexture {
                rendered_texture,
                modulation,
                opaque,
            },
            texture_view,
            encoder,
        );
    }
}

pub(crate) fn draw_glasses(
    st: &mut dyn ProgramState,
    char_info: &NxCharInfo,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let texture_header = st.texture_header();

    let texture = load_faceline_texture(
        st,
        char_info,
        texture_header.glass[char_info.glass_type as usize],
        ColorModulated::Glass,
    );

    let Some((rendered_texture, modulation)) = texture else {
        return;
    };

    st.draw_texture(
        DrawableTexture {
            rendered_texture,
            modulation,
            opaque: None,
        },
        texture_view,
        encoder,
    );
}

pub(crate) fn draw_hat(
    st: &mut dyn ProgramState,
    char_info: &NxCharInfo,
    texture_view: &TextureView,
    encoder: &mut CommandEncoder,
) {
    let res_texture = st.texture_header();

    let texture = load_faceline_texture(
        st,
        char_info,
        res_texture.hat[char_info.hair_type as usize],
        ColorModulated::Hat,
    );

    let Some((rendered_texture, modulation)) = texture else {
        return;
    };

    st.draw_texture(
        DrawableTexture {
            rendered_texture,
            modulation,
            opaque: None,
        },
        texture_view,
        encoder,
    );
}
