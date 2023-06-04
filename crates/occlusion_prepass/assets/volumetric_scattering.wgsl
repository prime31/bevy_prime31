#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var main_texture: texture_2d<f32>;

@group(1) @binding(1)
var main_sampler: sampler;

@group(1) @binding(2)
var occlusion_texture: texture_2d<f32>;

@group(1) @binding(3)
var occlusion_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // uniforms
    let samples = 100.0;
    let density = 0.98;
    let exposure = 0.18;
    let weight = 0.4;
    let decay = 0.96;
    let light_pos = vec2<f32>(0.5, 0.5);

    var uv = coords_to_viewport_uv(position.xy, view.viewport);
    var delta_uv = uv - light_pos;
    // Divide by number of samples and scale by control factor
    delta_uv *= 1.0 / f32(samples) * density;

    let diffuse = textureSample(main_texture, main_sampler, uv);
    var color = textureSample(occlusion_texture, occlusion_sampler, uv);
    var illumination_decay = 1.0;

    for (var i: i32 = 0; i < i32(samples); i++) {
        // step sample location along ray
        uv -= delta_uv;
        // retrieve sample at new location
        var sample = textureSample(occlusion_texture, occlusion_sampler, uv);
        // apply sample attenuation scale/decay factors
        sample *= illumination_decay * weight;
        // accumulate combined color
        color += sample;
        // update exponential decay factor
        illumination_decay *= decay;
    }

    //let offset_strength = 0.002;

    // Sample each color channel with an arbitrary shift
    //var output_color = vec4<f32>(
    //    textureSample(texture, main_sampler, uv + vec2<f32>(offset_strength, -offset_strength)).r,
    //    textureSample(texture, our_sampler, uv + vec2<f32>(-offset_strength, 0.0)).g,
    //    textureSample(texture, our_sampler, uv + vec2<f32>(0.0, offset_strength)).b,
    //    1.0
    //);

    // var occlusion_color = textureSample(occlusion_texture, occlusion_sampler, uv);
    // var output_color = textureSample(main_texture, main_sampler, uv);
    // return mix(output_color, occlusion_color, 0.5);
    // return mix(diffuse, color * exposure, 0.5);
    return diffuse + color * exposure;
    // return textureSample(main_texture, main_sampler, uv);
}
