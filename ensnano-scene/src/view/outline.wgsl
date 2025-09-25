struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0)
    );
    var o: VSOut;
    o.pos = vec4(pos[vid], 0.0, 1.0);
    o.uv = uv[vid];
    return o;
}

@group(0) @binding(0) var depth_tex_ms: texture_depth_multisampled_2d;

fn load_depth_ms(uv: vec2<f32>) -> f32 {
    let dims = textureDimensions(depth_tex_ms);
    let coord = vec2<i32>(clamp(
        vec2<i32>(vec2<f32>(dims) * uv),
        vec2<i32>(0),
        vec2<i32>(i32(dims.x) - 1, i32(dims.y) - 1)
    ));
    // TODO: linearize the depth
    return textureLoad(depth_tex_ms, coord, 0); // TODO: use MSAA
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // TODO: don't divide my dimensions just to multiply later. Find the pixel location in the texture instead.
    let step = 1.0 / vec2<f32>(textureDimensions(depth_tex_ms));

    let d00 = load_depth_ms(in.uv + vec2(-step.x, -step.y));
    let d10 = load_depth_ms(in.uv + vec2(0.0, -step.y));
    let d20 = load_depth_ms(in.uv + vec2(step.x, -step.y));
    let d01 = load_depth_ms(in.uv + vec2(-step.x, 0.0));
    let d21 = load_depth_ms(in.uv + vec2(step.x, 0.0));
    let d02 = load_depth_ms(in.uv + vec2(-step.x, step.y));
    let d12 = load_depth_ms(in.uv + vec2(0.0, step.y));
    let d22 = load_depth_ms(in.uv + vec2(step.x, step.y));

    // sobel filter
    let gx = (d20 + 2.0 * d21 + d22) - (d00 + 2.0 * d01 + d02);
    let gy = (d02 + 2.0 * d12 + d22) - (d00 + 2.0 * d10 + d20);
    let g = sqrt(gx * gx + gy * gy);

    return vec4(0.0, 0.0, 0.0, smoothstep(0.005, 0.02, g));
}
