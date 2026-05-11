#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    let world_matrix = get_world_from_local(0u);
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(world_matrix, vec4<f32>(position, 1.0));
    // Transform normal to world space (w=0 ignores translation).
    out.world_normal = normalize((world_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv = vertex.uv;
    out.color = vertex.i_color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let L = normalize(vec3<f32>(0.6, 1.0, 0.4));

    let diffuse = max(dot(N, L), 0.0);
    let ambient = 0.25;
    let brightness = ambient + diffuse * 0.75;

    // Subtle edge darkening: makes faces visually distinct within and between cubes.
    let edge_x = min(in.uv.x, 1.0 - in.uv.x);
    let edge_y = min(in.uv.y, 1.0 - in.uv.y);
    let edge = min(edge_x, edge_y);
    let edge_factor = smoothstep(0.0, 0.06, edge) * 0.2 + 0.8;

    let lit_color = in.color.rgb * brightness * edge_factor;
    return vec4<f32>(lit_color, in.color.a);
}
