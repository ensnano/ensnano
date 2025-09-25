struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Fullscreen triangle
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    let uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
    );

    return VertexOutput(
        vec4(pos[vi], 0.0, 1.0),
        uv[vi],
    );
}

@group(0) @binding(0) var depth_tex_ms: texture_depth_multisampled_2d;

// TODO: don't hardcode SAMPLE_COUNT = 4
fn load_depth(center: vec2<i32>, direction: vec2<i32>) -> f32 {
    let coord = clamp(center + direction, vec2<i32>(0), vec2<i32>(textureDimensions(depth_tex_ms) - 1));

    let d0 = textureLoad(depth_tex_ms, coord, 0);
    let d1 = textureLoad(depth_tex_ms, coord, 1);
    let d2 = textureLoad(depth_tex_ms, coord, 2);
    let d3 = textureLoad(depth_tex_ms, coord, 3);

    let min_depth = min(d0, min(d1, min(d2, d3)));
    let max_depth = max(d0, max(d1, max(d2, d3)));
    let median_depth = (d0 + d1 + d2 + d3 - min_depth - max_depth) / 2.0;

    // TODO: linearize the depth
    return median_depth;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<i32>(in.uv * vec2<f32>(textureDimensions(depth_tex_ms)));

    let d00 = load_depth(center, vec2<i32>(-1, -1));
    let d10 = load_depth(center, vec2<i32>(0, -1));
    let d20 = load_depth(center, vec2<i32>(1, -1));
    let d01 = load_depth(center, vec2<i32>(-1, 0));
    let d21 = load_depth(center, vec2<i32>(1, 0));
    let d02 = load_depth(center, vec2<i32>(-1, 1));
    let d12 = load_depth(center, vec2<i32>(0, 1));
    let d22 = load_depth(center, vec2<i32>(1, 1));

    // Sobel filter
    let gx = (d20 + 2.0 * d21 + d22) - (d00 + 2.0 * d01 + d02);
    let gy = (d02 + 2.0 * d12 + d22) - (d00 + 2.0 * d10 + d20);
    let g = sqrt(gx * gx + gy * gy);

    return vec4(0.0, 0.0, 0.0, smoothstep(0.005, 0.02, g));
}