struct TransformUniform {
    mtx: mat4x4<f32>,
    color_r: vec4<f32>,
    color_g: vec4<f32>,
    color_b: vec4<f32>,
    modulation_mode: u32,
};

@group(1) @binding(0) // 1.
var<uniform> mvp: TransformUniform;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) thirdthing: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = tex_coords;
    // if mvp.modulation_mode == 1 {
    //     // Ignore matrix with direct texture output.
    //     out.clip_position = vec4<f32>(position, 1.0);
    // } else {
        out.clip_position = mvp.mtx * vec4<f32>(position, 1.0);
    // }
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // if color.a == 0.0 {
    //    discard;
    // }

    // enum vfl::color::nx::ModulationMode {
    //     SingleColor = 0,
    //     DirectTexture = 1,
    //     LayeredRgbTexture = 2,
    //     AlphaTexture = 3,
    //     LuminanceAlphaTexture = 4,
    // }

    if mvp.modulation_mode == 0 {
        return modulate_single_color(color);
    } else if mvp.modulation_mode == 1 {
        return modulate_direct_texture(color);
    } else if mvp.modulation_mode == 2 {
        return modulate_rgba(color);
    } else if mvp.modulation_mode == 3 {
        return modulate_alpha(color);
    } else if mvp.modulation_mode == 4 {
        return modulate_lum_alpha(color);
    } else {
        // OOB access, return RebeccaPurple
        return vec4f(0.4, 0.2, 0.6, 1.0);
    }
}

// Two trivial cases
fn modulate_single_color(color: vec4f) -> vec4f {
    return mvp.color_r;
}

fn modulate_direct_texture(color: vec4f) -> vec4f {
    return color;
}

// Texture passes us alpha information, we fill in the rest of the color.
// [a,0,0,0] -> [r,g,b,a]
fn modulate_alpha(color: vec4f) -> vec4f {
    let repl = mvp.color_r;

    return vec4(repl.rgb, repl.a * color.r);
}

// Texture passes luminance + alpha, we colorize it.
// [l,a,0,0] -> [r,g,b,a]
fn modulate_lum_alpha(color: vec4f) -> vec4f {
    let repl_lum = mvp.color_r;

    return vec4(color.g * repl_lum.rgb, repl_lum.a * color.r);
}


fn modulate_rgba(color: vec4f) -> vec4f {
    let repl_r = mvp.color_r;
    let repl_g = mvp.color_g;
    let repl_b = mvp.color_b;

    return vec4f(color.r * repl_r.rgb + color.g * repl_g.rgb + color.b * repl_b.rgb, color.a * repl_r.a);

}
