struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct CharShapeUniform {
    color: vec4<f32>
}
@group(1) @binding(0)
var<uniform> char_shape: CharShapeUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    // instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    var world_position: vec4<f32> = vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    // Sampler AddressMode::MirrorRepeat + Move down one tex = Free Y-flip!
    let sample_position = in.tex_coords;
    let tex_color = textureSample(t_diffuse, s_diffuse, sample_position);

    // if tex_color.a == 0.0 {
    //    discard;
    // }

    // return tex_color;

    return mix(char_shape.color, tex_color, tex_color.a);

}
