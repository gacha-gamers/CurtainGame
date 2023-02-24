#import bevy_sprite::mesh2d_view_types

// NOTE: Bindings must come before functions that use them!
// #import bevy_sprite::mesh2d_functions

@group(0) @binding(0)
var<uniform> view: View;

struct PositionBuffer {
    data: array<vec4<f32>>
};
@group(1) @binding(0)
var<storage, read> positions: PositionBuffer;
struct RotationBuffer {
    data: array<vec2<f32>>
};
@group(1) @binding(1)
var<storage, read> rotations: RotationBuffer;

@group(2) @binding(0)
var bullet_texture: texture_2d<f32>;
@group(2) @binding(1)
var bullet_texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var vertex_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
    );

    var uvs: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    
    let bullet_id = in_vertex_index / 6u;
    let vertex_id = in_vertex_index % 6u;

    let bullet_pos = positions.data[bullet_id].xy;
    let bullet_rot = rotations.data[bullet_id].x;

    let cos_sin = vec2<f32>(cos(bullet_rot), sin(bullet_rot));
    let rot_matrix = mat2x2<f32>(
        cos_sin.y, cos_sin.x, 
        -cos_sin.x, cos_sin.y);
    
    var vertex_pos = vertex_positions[vertex_id];
    vertex_pos = vertex_pos * rot_matrix * 16.0;

    var out: VertexOutput;
    // Calculate the UV from the pattern [[0,0], [1,0], [0,1], [1,1]]
    //out.uv = vec2<f32>(f32(in_vertex_index & 1u), f32((in_vertex_index & 2u) / 2u));
    out.uv = uvs[vertex_id];
    out.position = view.view_proj * vec4<f32>(bullet_pos + vertex_pos, 0.0, 1.0);
    return out;
}

// @group(2) @binding(0)
// var texture: texture_2d<f32>;
// @group(2) @binding(1)
// var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(bullet_texture, bullet_texture_sampler, in.uv);
}
