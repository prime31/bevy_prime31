#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings


struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return in.color;
}
