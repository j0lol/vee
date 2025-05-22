struct TransformUniform {
    mtx: mat4x4<f32>,
    color_r: vec4<f32>,
    color_g: vec4<f32>,
    color_b: vec4<f32>,
    texture_format: u32,
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
    out.clip_position = mvp.mtx * vec4<f32>(position, 1.0);
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

    // R = 0,       // R8Unorm (Ffl Name)
    // Rb = 1,      // R8B8Unorm
    // Rgba = 2,    // R8B8G8A8Unorm
    // Bc4 = 3,     // Bc4Unorm (Compressed R)
    // Bc5 = 4,     // Bc5Unorm (Compressed Rb)
    // Bc7 = 5,     // Bc7Unorm (Compressed Rgba)
    // Astc4x4 = 6, // Astc4x4Unorm (Compressed Rgba)

    // Luminance Alpha ∈ [1,4]
    // Rgba ∈ [2,5,6]
    // Alpha ∈ [0,3]

    if (mvp.texture_format == 1 || mvp.texture_format == 4) {
        return normalize_lum_alpha(color);
    } else if (mvp.texture_format == 2 || mvp.texture_format == 5 || mvp.texture_format == 6) {
        return normalize_rgba(color);
    } else if (mvp.texture_format == 0 || mvp.texture_format == 3) {
        return normalize_alpha(color);
    } else {
        return vec4f(0.4, 0.2, 0.6, 1.0); // Rebecca Purple
    }

    return color;
}

fn normalize_rgba(color: vec4f) -> vec4f {
    let repl_r = mvp.color_r;
    let repl_g = mvp.color_g;
    let repl_b = mvp.color_b;

    return vec4f(color.r * repl_r.rgb + color.g * repl_g.rgb + color.b * repl_b.rgb, color.a * repl_r.a);

}

// Texture passes luminance + alpha, we colorize it.
fn normalize_lum_alpha(color: vec4f) -> vec4f {
    let repl_lum = mvp.color_r;

    return vec4(color.g * repl_lum.rgb, repl_lum.a * color.r);
}

// Texture passes us alpha information, we fill in the rest of the color.
fn normalize_alpha(color: vec4f) -> vec4f {
    let repl = mvp.color_r;

    return vec4(repl.rgb, repl.a * color.r);
}
