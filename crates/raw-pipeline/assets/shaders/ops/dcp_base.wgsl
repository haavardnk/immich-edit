fn dcp_base_apply(c: vec3<f32>) -> vec3<f32> {
    let dims = vec4<u32>(p.dcp_base[6]);
    if (dims.x == 0u) {
        return c;
    }
    let pp = vec3<f32>(
        dot(p.dcp_base[0].xyz, c),
        dot(p.dcp_base[1].xyz, c),
        dot(p.dcp_base[2].xyz, c),
    );
    let mapped = huesat_map(dcp_base_tex, dims, pp, false);
    return vec3<f32>(
        dot(p.dcp_base[3].xyz, mapped),
        dot(p.dcp_base[4].xyz, mapped),
        dot(p.dcp_base[5].xyz, mapped),
    );
}
