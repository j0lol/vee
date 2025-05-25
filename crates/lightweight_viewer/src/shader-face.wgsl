struct TransformUniform {
    mtx: mat4x4<f32>,
    color_r: vec4<f32>,
    color_g: vec4<f32>,
    color_b: vec4<f32>,
    texture_format: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.clip_position = vec4<f32>(position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords - vec2(1.0, 0.0));

    if color.a == 0.0 {
       discard;
    }

    return color;
}
