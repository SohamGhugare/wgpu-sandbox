@group(0) @binding(0) var<uniform> u_color: vec4<f32>;
@group(0) @binding(1) var<uniform> u_aspect: vec4<f32>; // x = aspect ratio (width / height)

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    let corrected = vec2<f32>(position.x / u_aspect.x, position.y);
    return vec4<f32>(corrected, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return u_color;
}
