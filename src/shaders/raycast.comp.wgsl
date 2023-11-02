struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_to_world: mat4x4<f32>,
    eye: vec3<f32>,
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

@group(2) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

@group(3) @binding(0)
var node5s: texture_3d<f32>;
@group(3) @binding(1)
var node4s: texture_3d<f32>;
@group(3) @binding(2)
var node3s: texture_3d<f32>;

@compute @workgroup_size(8,4)
fn cp_main(@builtin(global_invocation_id) global_id : vec3<u32>) {
    var p = vec2<f32>(global_id.xy) + vec2(0.001);
    var ray_dir = normalize((p.x * r.u + p.y * r.mv + r.wp).xyz);
    var color = vec4(cast_ray(camera.eye, ray_dir),1.0);
    var val = textureLoad(node5s, vec3<i32>(0, 0, 0), 0);
    // We got this from the VDB!!
    //  Shader madness incoming

    textureStore(texture, global_id.xy, color);
}
const MAX_RAY_STEPS: i32 = 128;
fn cast_ray(src: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var ipos = vec3<i32>(floor(src));
    var deltaDist = abs(vec3<f32>(length(dir)) / dir);
    var step = vec3<i32>(sign(dir));
    var sideDist = (sign(dir) * (vec3<f32>(ipos) - src) + (sign(dir) * 0.5) + 0.5) * deltaDist;
    var mask = vec3<bool>(false);
    var i: i32 = 0;

    for (i = 0; i < MAX_RAY_STEPS; i++) {
        if (getVoxel(ipos)) {
            break;
        }

        var b1 = sideDist.xyz <= sideDist.yzx;
        var b2 = sideDist.xyz <= sideDist.zxy;
        mask = b1 & b2;

        sideDist += vec3<f32>(mask) * deltaDist;
        ipos += vec3<i32>(mask) * step;
    }

    if (i == MAX_RAY_STEPS) {
        return vec3<f32>(dir);
    }
    if (mask.x) {
        return vec3<f32>(0.5);
    }
    if (mask.y) {
        return vec3<f32>(1.0);
    }
    if (mask.z) {
        return vec3<f32>(0.75);
    }
    return vec3<f32>(0.3);
}

fn getVoxel(pos: vec3<i32>) -> bool{
    return ( ( pos.x == pos.z || pos.x == -pos.z ) && pos.y == 0 && (pos.x == 4 || pos.x == 2 || pos.x == 6));
}
