struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_to_world: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Input {
    @location(0) position: vec3<f32>,
}

// Vertex shader
@vertex
fn vs_main(model: Input) -> @builtin(position) vec4<f32> {
    //out.clip_position = camera.view_proj * world_vert;

    return vec4<f32>(model.position, 1.0);
}
