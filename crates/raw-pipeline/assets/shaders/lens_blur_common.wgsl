struct Params {
    size: vec2<u32>,
    origin: vec2<u32>,
    full: vec2<u32>,
    grid_origin: vec2<u32>,
    grid_size: vec2<u32>,
    ds: u32,
    reach: i32,
    orient: vec3<u32>,
    lens: vec4<f32>,
    radius: f32,
    boost: f32,
    blades: u32,
    table: array<vec4<f32>, COC_LEVELS / 4u>,
};

@group(0) @binding(0) var<uniform> p: Params;

fn oriented_dims() -> vec2<u32> {
    if (p.orient.x != 0u) {
        return p.full.yx;
    }
    return p.full;
}

fn sensor_to_oriented(s: vec2<u32>) -> vec2<u32> {
    var f = s;
    if (p.orient.y != 0u) {
        f.x = p.full.x - 1u - s.x;
    }
    if (p.orient.z != 0u) {
        f.y = p.full.y - 1u - s.y;
    }
    if (p.orient.x != 0u) {
        return f.yx;
    }
    return f;
}

fn oriented_to_sensor(o: vec2<u32>) -> vec2<u32> {
    var f = o;
    if (p.orient.x != 0u) {
        f = o.yx;
    }
    if (p.orient.y != 0u) {
        f.x = p.full.x - 1u - f.x;
    }
    if (p.orient.z != 0u) {
        f.y = p.full.y - 1u - f.y;
    }
    return f;
}

fn table_value(i: u32) -> f32 {
    return p.table[i / 4u][i % 4u];
}

fn coc_table(d: f32) -> f32 {
    let f = clamp(d, 0.0, 1.0) * f32(COC_LEVELS - 1u);
    let i = min(u32(floor(f)), COC_LEVELS - 2u);
    let t = f - f32(i);
    let v0 = table_value(i);
    return v0 + (table_value(i + 1u) - v0) * t;
}
