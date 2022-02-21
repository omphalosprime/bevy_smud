#import bevy_smud::prelude
#import bevy_smud::colorize

struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

struct Time {
    time_since_startup: f32;
};
[[group(1), binding(0)]]
var<uniform> time: Time;
// as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] rotation: vec2<f32>;
    [[location(3)]] scale: f32;
    [[location(4)]] frame: f32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] pos: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(
    vertex: Vertex,
    [[builtin(vertex_index)]] i: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x = select(-1., 1., i % 2u == 0u);
    let y = select(-1., 1., (i / 2u) % 2u == 0u);
    let c = vertex.rotation.x;
    let s = vertex.rotation.y;
    let rotated = vec2<f32>(x * c - y * s, x * s + y * c);
    // let rotated = vec2<f32>(x, y);
    // let w = 400.;
    // let w = 80.;
    let pos = vertex.position + vec3<f32>(rotated * vertex.scale * vertex.frame, vertex.position.z);
    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * vec4<f32>(pos, 1.);
    out.color = vertex.color;
        out.color.r = time.time_since_startup / 10.;
    out.pos = vec2<f32>(x, y) * vertex.frame;
    return out;
}