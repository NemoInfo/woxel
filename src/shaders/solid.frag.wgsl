struct Input {
    @builtin(position) position: vec4<f32>,
}

struct Output {
    @location(0) color: vec4<f32>,
};

@fragment
fn fs_main(frag: Input) -> @location(0) vec4<f32>
{
    var valr: f32 = frag.position.x / 1600.0;
    var valg: f32 = 1.0 - valr;
    return vec4<f32>(valr, valg, 0.0, 1.0);
}
