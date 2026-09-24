struct Out {
    @builtin(position) pos: vec4<f32>,
}

@group(0) @binding(0) var src: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> Out {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return Out(vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0));
}

@fragment
fn fs(input: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(textureLoad(src, vec2<i32>(input.pos.xy), 0).rgb, 1.0);
}
