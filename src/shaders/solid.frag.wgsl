struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_to_world: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct RayUniform {
    u: vec4<f32>,
    mv: vec4<f32>,
    wp: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> r: RayUniform;

struct Input {
    @builtin(position) position: vec4<f32>,
}

struct Output {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(frag: Input) -> @location(0) vec4<f32>
{
    var p: vec2<f32> = vec2<f32>(frag.position.xy);
    var ray_dir = normalize(p.x * r.u + p.y * r.mv + r.wp);
    var valr: f32 = frag.position.x / 1600.0;
    var valg: f32 = 1.0 - valr;
    return vec4<f32>(ray_dir.xyz, 1.0);
}
