@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tsampler: sampler;

struct Input {
    @builtin(position) position: vec4<f32>,
}

struct Output {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(frag: Input) -> @location(0) vec4<f32>
{
    var p = vec2<f32>(frag.position.xy) / vec2(1600.0, 900.0);
    var color = textureSample(texture, tsampler, p);

    return color;
}
