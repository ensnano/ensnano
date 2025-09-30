struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Fullscreen triangle
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var uv = array<vec2<f32>, 3>(
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

struct OutlineUniform {
    sample_count: u32,
    use_outline: u32,
    camera_near: f32,
    camera_far: f32,
};

@group(0) @binding(1) var<uniform> u_outline: OutlineUniform;

// Linearize WebGPU depth in [0, 1] to a camera space z in [0, ∞]
fn linearize_depth(depth: f32) -> f32 {
    let n = u_outline.camera_near;
    let f = u_outline.camera_far;
    return (n * f) / (f - depth * (f - n));
}

// TODO: don't hardcode sample count (4), camera near (0.1) and camera far (1000.0)
fn load_depth(center: vec2<i32>, direction: vec2<i32>) -> f32 {
    let coord = clamp(
        center + direction,
        vec2<i32>(0),
        vec2<i32>(textureDimensions(depth_tex_ms)) - 1
    );

    let d0 = textureLoad(depth_tex_ms, coord, 0);
    var min_d = d0;
    var max_d = d0;
    var sum_d = d0;

    for (var i = 1u; i < u_outline.sample_count; i = i + 1u) {
        let di = textureLoad(depth_tex_ms, coord, i32(i));
        sum_d = sum_d + di;
        min_d = min(min_d, di);
        max_d = max(max_d, di);
    }

    var truncated_mean_depth: f32;
    if u_outline.sample_count > 2 {
        truncated_mean_depth = (sum_d - min_d - max_d) / f32(u_outline.sample_count - 2);
    } else {
        truncated_mean_depth = sum_d / f32(u_outline.sample_count);
    }

    return linearize_depth(truncated_mean_depth);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<i32>(in.uv * vec2<f32>(textureDimensions(depth_tex_ms)));

    // Don't outline the skybox
    let d11 = load_depth(center, vec2<i32>(0, 0));
    if d11 > 300.0 { // TODO: get skybox size (500) in uniform
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let step = 1; // TODO: step varies smoothly based on distance

    let d00 = load_depth(center, vec2<i32>(-step, -step));
    let d10 = load_depth(center, vec2<i32>(0, -step));
    let d20 = load_depth(center, vec2<i32>(step, -step));
    let d01 = load_depth(center, vec2<i32>(-step, 0));
    let d21 = load_depth(center, vec2<i32>(step, 0));
    let d02 = load_depth(center, vec2<i32>(-step, step));
    let d12 = load_depth(center, vec2<i32>(0, step));
    let d22 = load_depth(center, vec2<i32>(step, step));

    // Sobel filter
    let gx = (d20 + 2.0 * d21 + d22) - (d00 + 2.0 * d01 + d02);
    let gy = (d02 + 2.0 * d12 + d22) - (d00 + 2.0 * d10 + d20);
    // Gradient squared, to avoid costly sqrt
    let g2 = gx * gx + gy * gy;

    return select(
        vec4(vec3(smoothstep(4.0, 1.0, g2)), 1.0),
        vec4(0.0, 0.0, 0.0, smoothstep(1.0, 4.0, g2)),
        vec4(u_outline.use_outline == 1u)
    );
}