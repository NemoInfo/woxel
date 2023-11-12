struct Camera {
    view_proj: mat4x4<f32>,
    camera_to_world: mat4x4<f32>,
    eye: vec3<f32>,
};

struct Ray {
    u: vec4<f32>,
    mv: vec4<f32>,
    wp: vec4<f32>,
};

struct State {
    camera: Camera,
    ray: Ray,
};
@group(0) @binding(0)
var<uniform> s: State;

@group(1) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

@group(2) @binding(0)
var node5s: texture_3d<u32>;
@group(2) @binding(1)
var node4s: texture_3d<u32>;
@group(2) @binding(2)
var node3s: texture_3d<u32>;

struct Node5Mask {
    m: array<u32, 1024>, // 32^3/32
};

struct Node4Mask {
    m: array<u32, 128>, // 16^3/32
};

struct Node3Mask {
    m: array<u32, 16>, // 8^3/32
};

@group(3) @binding(0)
var<storage, read> kids5: array<Node5Mask>;
@group(3) @binding(1)
var<storage, read> vals5: array<Node5Mask>;
@group(3) @binding(2)
var<storage, read> kids4: array<Node4Mask>;
@group(3) @binding(3)
var<storage, read> vals4: array<Node4Mask>;
@group(3) @binding(4)
var<storage, read> vals3: array<Node3Mask>;
@group(3) @binding(5)
var<storage, read> origins: array<vec3<i32>>;

@compute @workgroup_size(8,4)
fn cp_main(@builtin(global_invocation_id) global_id : vec3<u32>) {
    var p = vec2<f32>(global_id.xy) + vec2(0.001);
    var ray_dir = normalize((p.x * s.ray.u + p.y * s.ray.mv + s.ray.wp).xyz);
    var color = vec4(hdda(s.camera.eye, ray_dir),1.0);
    var val = textureLoad(node5s, vec3<i32>(0, 0, 0), 0);

    textureStore(texture, global_id.xy, color);
}

const HDDA_MAX_RAY_STEPS: i32 = 1500;
fn hdda(src: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var ipos = vec3<i32>(floor(src));
    var deltaDist = abs(vec3<f32>(length(dir)) / dir);
    var step = vec3<i32>(sign(dir));
    var sideDist = (sign(dir) * (vec3<f32>(ipos) - src) + (sign(dir) * 0.5) + 0.5) * deltaDist;
    var mask = vec3<bool>(false);

    var i: i32 = 0;
    var c = dir;

    var leaf: VdbLeaf;
    leaf.empty = true;

    for(i = 0; i < HDDA_MAX_RAY_STEPS; i++) {
        // FIND VBD LEAF
        // RECORD LEVEL
        // IF VALUE RETURN
        // IF EMPTY STEP * LEVEL
        //
        leaf = get_vdb_leaf_from_leaf(ipos, leaf);
        if !leaf.empty {
            return leaf.color + vec3<f32>(0.25, 0.5, 0.70) * vec3<f32>(mask);
            // if (mask.x) {
            //     return vec3<f32>(0.25) + leaf.color;
            // }
            // if (mask.y) {
            //     return vec3<f32>(0.50) + leaf.color;
            // }
            // if (mask.z) {
            //     return vec3<f32>(0.70) + leaf.color;
            // }
            // return vec3<f32>(0.3);
        }

        // choose which direction is the smallest
        var b1 = sideDist.xyz <= sideDist.yzx;
        var b2 = sideDist.xyz <= sideDist.zxy;
        mask = b1 & b2;

        sideDist += vec3<f32>(mask) * deltaDist;
        ipos += vec3<i32>(mask) * step * 1;
    }

    return vec3<f32>(dir);

}

struct Parent {
    origin: vec3<i32>,
    idx: u32,
}

struct VdbLeaf {
    color: vec3<f32>,
    empty: bool,
    num_parents: u32,
    parents: array<Parent, 3>,
}

fn get_vdb_leaf_from_leaf(pos: vec3<i32>, leaff: VdbLeaf) -> VdbLeaf {
    var leaf = leaff;
    if leaf.num_parents == 3u {
        let node3_global = global_to_node(pos, NODE3_TOTAL_LOG_D);
        if all(leaf.parents[2].origin == node3_global) {
            return get_vdb_leaf_from_node3(pos, leaf);
        }
        let node4_global = global_to_node(pos, NODE4_TOTAL_LOG_D);
        if all(leaf.parents[1].origin == node4_global) {
            return get_vdb_leaf_from_node4(pos, leaf);
        }
        let node5_global = global_to_node(pos, NODE5_TOTAL_LOG_D);
        if all(leaf.parents[0].origin == node4_global) {
            return get_vdb_leaf_from_node5(pos, leaf);
        }
        return get_vdb_leaf_from_nothing(pos, leaf);
    }
    if leaf.num_parents == 2u {
        let node4_global = global_to_node(pos, NODE4_TOTAL_LOG_D);
        if all(leaf.parents[1].origin == node4_global) {
            return get_vdb_leaf_from_node4(pos, leaf);
        }
        let node5_global = global_to_node(pos, NODE5_TOTAL_LOG_D);
        if all(leaf.parents[0].origin == node4_global) {
            return get_vdb_leaf_from_node5(pos, leaf);
        }
        return get_vdb_leaf_from_nothing(pos, leaf);
    }
    if leaf.num_parents == 1u {
        let node5_global = global_to_node(pos, NODE5_TOTAL_LOG_D);
        if all(leaf.parents[0].origin == node5_global) {
            return get_vdb_leaf_from_node5(pos, leaf);
        }
        return get_vdb_leaf_from_nothing(pos, leaf);
    }
    return get_vdb_leaf_from_nothing(pos, leaf);
}

fn get_vdb_leaf_from_nothing(pos: vec3<i32>, leaff: VdbLeaf) -> VdbLeaf {
    var leaf = leaff;
    let node5_global = global_to_node(pos, NODE5_TOTAL_LOG_D);

    for (var node5_idx: u32 =0u; node5_idx < arrayLength(&origins); node5_idx++) {
        if all(node5_global == origins[node5_idx]) {
            leaf.parents[0] = Parent(node5_global, node5_idx);
            leaf.num_parents = 1u;

            return get_vdb_leaf_from_node5(pos, leaf);
        }
    }

    return VdbLeaf(vec3<f32>(0.0), true, 0u, leaf.parents);
}

fn get_vdb_leaf_from_node5(pos: vec3<i32>, leaff: VdbLeaf) -> VdbLeaf {
    var leaf = leaff;
    let node5_local = global_to_local(pos, NODE5_TOTAL_LOG_D);
    let node5_child = local_to_child_node(node5_local, NODE4_TOTAL_LOG_D);
    let node5_offset = child_to_offset(node5_child, 5u, 10u);
    let node5_idx = leaf.parents[0].idx;

    let node5_mask_index = node5_offset >> 5u;
    let node5_mask_pos = node5_offset & 31u;
    let in_kid5 = bool( kids5[node5_idx].m[node5_mask_index] & ( 1u << node5_mask_pos));
    let in_val5 = bool( vals5[node5_idx].m[node5_mask_index] & ( 1u << node5_mask_pos));

    let node5_atlas_dim = textureDimensions(node5s).y >> 5u;
    let node5_atlas_origin = 32u * atlas_origin_from_idx(node5_idx, node5_atlas_dim);
    let node4_idx = textureLoad(node5s, node5_child + node5_atlas_origin, 0).r;

    if (in_val5) {
        return VdbLeaf(vec3<f32>(0.2), false , 1u, leaf.parents);
    }

    if (!in_kid5) {
        return VdbLeaf(vec3<f32>(0.0), true , 1u, leaf.parents);
    }

    let node4_global = global_to_node(pos, NODE4_TOTAL_LOG_D);
    leaf.parents[1] = Parent(node4_global, node4_idx);
    leaf.num_parents = 2u;

    return get_vdb_leaf_from_node4(pos, leaf);
}

fn get_vdb_leaf_from_node4(pos: vec3<i32>, leaff: VdbLeaf) -> VdbLeaf {
    var leaf = leaff;
    let node4_local = global_to_local(pos, NODE4_TOTAL_LOG_D);
    let node4_child = local_to_child_node(node4_local, NODE3_TOTAL_LOG_D);
    let node4_offset = child_to_offset(node4_child, 4u, 8u);
    let node4_idx = leaf.parents[1].idx;

    let node4_mask_index = node4_offset >> 5u;
    let node4_mask_pos = node4_offset & 31u;
    let in_kid4 = bool( kids4[node4_idx].m[node4_mask_index] & ( 1u << node4_mask_pos));
    let in_val4 = bool( vals4[node4_idx].m[node4_mask_index] & ( 1u << node4_mask_pos));


    let node4_atlas_dim = textureDimensions(node4s).x >> 4u;
    let node4_atlas_origin = 16u * atlas_origin_from_idx(node4_idx, node4_atlas_dim);
    let node3_idx = textureLoad(node4s, node4_child + node4_atlas_origin, 0).r;

    if (in_val4) {
        return VdbLeaf(vec3<f32>(0.2), false , 2u, leaf.parents);
    }
    if (!in_kid4) {
        return VdbLeaf(vec3<f32>(0.0), true , 2u, leaf.parents);
    }

    let node3_global = global_to_node(pos, NODE3_TOTAL_LOG_D);
    leaf.parents[2] = Parent(node3_global, node3_idx);
    leaf.num_parents = 3u;

    return get_vdb_leaf_from_node3(pos, leaf);
}

fn get_vdb_leaf_from_node3(pos: vec3<i32>, leaff: VdbLeaf) -> VdbLeaf {
    var leaf = leaff;
    let node3_local = global_to_local(pos, NODE3_TOTAL_LOG_D);
    let node3_offset = child_to_offset(node3_local, 3u, 6u);
    let node3_idx = leaf.parents[2].idx;

    let node3_mask_index = node3_offset >> 5u;
    let node3_mask_pos = node3_offset & 31u;
    let in_val3 = bool( vals3[node3_idx].m[node3_mask_index] & ( 1u << node3_mask_pos));

    let node3_atlas_dim = textureDimensions(node3s).x >> 3u;
    let node3_atlas_origin = 8u * atlas_origin_from_idx(node3_idx, node3_atlas_dim);
    let voxel = textureLoad(node3s, node3_local + node3_atlas_origin, 0).r;
    if (in_val3) {
        return VdbLeaf(vec3<f32>(0.1), false, 3u, leaf.parents);
    }
    return VdbLeaf(vec3<f32>(0.0), true, 3u, leaf.parents);
}

const MAX_RAY_STEPS: i32 = 1500;
fn cast_ray(src: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var ipos = vec3<i32>(floor(src));
    var deltaDist = abs(vec3<f32>(length(dir)) / dir);
    var step = vec3<i32>(sign(dir));
    var sideDist = (sign(dir) * (vec3<f32>(ipos) - src) + (sign(dir) * 0.5) + 0.5) * deltaDist;
    var mask = vec3<bool>(false);
    var i: i32 = 0;
    var c = dir;

    for (i = 0; i < MAX_RAY_STEPS; i++) {
        c = getVdbVoxel(ipos);
        if (c.x != 0.33) {
            break;
        }

        // choose which direction is the smallest
        var b1 = sideDist.xyz <= sideDist.yzx;
        var b2 = sideDist.xyz <= sideDist.zxy;
        mask = b1 & b2;

        sideDist += vec3<f32>(mask) * deltaDist;
        ipos += vec3<i32>(mask) * step * 1;
    }

    if (i == MAX_RAY_STEPS) {
        return vec3<f32>(dir);
    }
    if (mask.x) {
        return vec3<f32>(0.25) + c;
    }
    if (mask.y) {
        return vec3<f32>(0.50) + c;
    }
    if (mask.z) {
        return vec3<f32>(0.70) + c;
    }
    return vec3<f32>(0.3);
}

fn getVoxel(pos: vec3<i32>) -> bool{
    return ( ( pos.x == pos.z || pos.x == -pos.z ) && pos.y == 0 && (pos.x == 4 || pos.x == 2 || pos.x == 6));
}


const NODE5_TOTAL_LOG_D: u32 = 12u; // 5 + 4 + 3
const NODE4_TOTAL_LOG_D: u32 = 7u; // 4 + 3
const NODE3_TOTAL_LOG_D: u32 = 3u; // 3

fn getVdbVoxel(pos: vec3<i32>) -> vec3<f32> {
    var node5_origin = global_to_node(pos, NODE5_TOTAL_LOG_D);

    for (var node5_idx: u32 =0u; node5_idx < arrayLength(&origins); node5_idx++) {
        if all(node5_origin == origins[node5_idx]) {

            let node5_local = global_to_local(pos, NODE5_TOTAL_LOG_D);
            let node5_child = local_to_child_node(node5_local, NODE4_TOTAL_LOG_D);
            let node5_offset = child_to_offset(node5_child, 5u, 10u);

            let node5_mask_index = node5_offset >> 5u;
            let node5_mask_pos = node5_offset & 31u;
            let in_kid5 = bool( kids5[node5_idx].m[node5_mask_index] & ( 1u << node5_mask_pos));
 //           let in_val5 = bool( vals5[node5_idx].m[node5_mask_index] & ( 1u << node5_mask_pos));


            if (in_kid5) {
                let node5_atlas_dim = textureDimensions(node5s).y >> 5u;

                let node5_atlas_origin = 32u * atlas_origin_from_idx(node5_idx, node5_atlas_dim);
                let node4_idx = textureLoad(node5s, node5_child + node5_atlas_origin, 0).r;

                let node4_local = global_to_local(pos, NODE4_TOTAL_LOG_D);
                let node4_child = local_to_child_node(node4_local, NODE3_TOTAL_LOG_D);
                let node4_offset = child_to_offset(node4_child, 4u, 8u);

                let node4_mask_index = node4_offset >> 5u;
                let node4_mask_pos = node4_offset & 31u;
                let in_kid4 = bool( kids4[node4_idx].m[node4_mask_index] & ( 1u << node4_mask_pos));
//                let in_val4 = bool( vals4[node4_idx].m[node4_mask_index] & ( 1u << node4_mask_pos));

                if (in_kid4) {
                    let node4_atlas_dim = textureDimensions(node4s).x >> 4u;
                    let node4_atlas_origin = 16u * atlas_origin_from_idx(node4_idx, node4_atlas_dim);
                    let node3_idx = textureLoad(node4s, node4_child + node4_atlas_origin, 0).r;

                    let node3_local = global_to_local(pos, NODE3_TOTAL_LOG_D);
                    let node3_offset = child_to_offset(node3_local, 3u, 6u);

                    let node3_mask_index = node3_offset >> 5u;
                    let node3_mask_pos = node3_offset & 31u;
                    let in_val3 = bool( vals3[node3_idx].m[node3_mask_index] & ( 1u << node3_mask_pos));

                    if (in_val3) {
                        return vec3<f32>(0.1);
                    }
                    break;
                }
                break;
            }
            break;
        }
    }

    return vec3<f32>(0.33, 0.0, 0.0);
}


fn global_to_node(pos: vec3<i32>, total_log_d: u32) -> vec3<i32> {
    // This are the global coordinates of a node that contains position
    return (pos >> total_log_d) << total_log_d;
}

fn global_to_local(pos: vec3<i32>, total_log_d: u32) -> vec3<u32> {
    // This are the relative coordinates of a voxel inside of a node
    return vec3<u32>(pos & ((1 << total_log_d) - 1));
}

fn local_to_child_node(pos: vec3<u32>, child_total_log_d: u32) -> vec3<u32> {
    // This are the relative coordinates of a child inside of a node
    return pos >> child_total_log_d;
}

fn child_to_offset(pos: vec3<u32>, log_d: u32, log_dd: u32) -> u32 {
    return (pos.x << log_dd) | (pos.y << log_d) | pos.z;
}

fn atlas_origin_from_idx(idx: u32, dim: u32) -> vec3<u32> {
    // Return node origin in atlas
    return vec3(idx % dim, (idx / dim) % dim, idx / (dim * dim));
}
