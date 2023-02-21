#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_sprite::mesh2d_functions

struct Vertex {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};


@vertex
fn vertex(vertex: Vertex, @builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = mesh2d_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0, 1.0));
    out.clip_position = mesh2d_position_world_to_clip(out.world_position);
    // Calculate the UV from the pattern [[0,0], [1,0], [0,1], [1,1]]
    out.uv = vec2<f32>(f32(in_vertex_index & 1u), f32((in_vertex_index & 2u) / 2u));
    return out;
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var our_sampler: sampler;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = textureSample(texture, our_sampler, input.uv);
    return output_color;
}
