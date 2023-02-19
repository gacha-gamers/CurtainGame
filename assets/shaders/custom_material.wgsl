#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_sprite::mesh2d_functions

// struct CustomMaterial {
//     color: vec4<f32>,
// }

// @group(1) @binding(0)
// var<uniform> material: CustomMaterial;
// @group(1) @binding(1)
// var color_texture: texture_2d<f32>;
// @group(1) @binding(2)
// var color_sampler: sampler;


struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct MyVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};


@vertex
fn vertex(vertex: Vertex) -> MyVertexOutput {
    var out: MyVertexOutput;

    out.world_position = mesh2d_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh2d_position_world_to_clip(out.world_position);
    out.uv = vertex.uv;
    return out;
}

struct MyMat {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> uniform_data: MyMat;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var our_sampler: sampler;

@fragment
fn fragment(input: MyVertexOutput) -> @location(0) vec4<f32> {
    var output_color = vec4<f32>(0.5, 1.0, 1.0, 1.0);
    output_color = output_color * textureSample(texture, our_sampler, input.uv);
    // output_color = output_color * uniform_data.color;
    return output_color;
}
