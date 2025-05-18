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

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return char_shape.color;
    // let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // // We don't need (or want) much ambient light, so 0.1 is fine
    // let ambient_strength = 0.1;
    // let ambient_color = light.color * ambient_strength;

    // let light_dir = normalize(light.position - in.world_position);
    // let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    // let half_dir = normalize(view_dir + light_dir);

    // let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    // let diffuse_color = light.color * diffuse_strength;

    // let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
    // let specular_color = specular_strength * light.color;

    // let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    // return vec4<f32>(result, object_color.a);
}
