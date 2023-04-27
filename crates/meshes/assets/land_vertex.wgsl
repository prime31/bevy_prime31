#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn taylorInvSqrt4(r: vec4<f32>) -> vec4<f32> { return 1.79284291400159 - 0.85373472095314 * r; }

fn simplexNoise3(v: vec3<f32>) -> f32 {
  let C = vec2<f32>(1. / 6., 1. / 3.);
  let D = vec4<f32>(0., 0.5, 1., 2.);

  // First corner
  var i: vec3<f32>  = floor(v + dot(v, C.yyy));
  let x0 = v - i + dot(i, C.xxx);

  // Other corners
  let g = step(x0.yzx, x0.xyz);
  let l = 1.0 - g;
  let i1 = min(g.xyz, l.zxy);
  let i2 = max(g.xyz, l.zxy);

  // x0 = x0 - 0. + 0. * C
  let x1 = x0 - i1 + 1. * C.xxx;
  let x2 = x0 - i2 + 2. * C.xxx;
  let x3 = x0 - 1. + 3. * C.xxx;

  // Permutations
  i = i % vec3<f32>(289.);
  let p = permute4(permute4(permute4(
      i.z + vec4<f32>(0., i1.z, i2.z, 1. )) +
      i.y + vec4<f32>(0., i1.y, i2.y, 1. )) +
      i.x + vec4<f32>(0., i1.x, i2.x, 1. ));

  // Gradients (NxN points uniformly over a square, mapped onto an octahedron.)
  var n_: f32 = 1. / 7.; // N=7
  let ns = n_ * D.wyz - D.xzx;

  let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

  let x_ = floor(j * ns.z);
  let y_ = floor(j - 7.0 * x_); // mod(j, N)

  let x = x_ *ns.x + ns.yyyy;
  let y = y_ *ns.x + ns.yyyy;
  let h = 1.0 - abs(x) - abs(y);

  let b0 = vec4<f32>( x.xy, y.xy );
  let b1 = vec4<f32>( x.zw, y.zw );

  let s0 = floor(b0)*2.0 + 1.0;
  let s1 = floor(b1)*2.0 + 1.0;
  let sh = -step(h, vec4<f32>(0.));

  let a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
  let a1 = b1.xzyw + s1.xzyw*sh.zzww ;

  var p0: vec3<f32> = vec3<f32>(a0.xy, h.x);
  var p1: vec3<f32> = vec3<f32>(a0.zw, h.y);
  var p2: vec3<f32> = vec3<f32>(a1.xy, h.z);
  var p3: vec3<f32> = vec3<f32>(a1.zw, h.w);

  // Normalise gradients
  let norm = taylorInvSqrt4(vec4<f32>(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
  p0 = p0 * norm.x;
  p1 = p1 * norm.y;
  p2 = p2 * norm.z;
  p3 = p3 * norm.w;

  // Mix final noise value
  var m: vec4<f32> = 0.6 - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3));
  m = max(m, vec4<f32>(0.));
  m = m * m;
  return 42. * dot(m * m, vec4<f32>(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

struct StandardMaterial {
    time: f32,
};

@group(1) @binding(0)
var<uniform> material: StandardMaterial;
@group(1) @binding(1)
var<uniform> ship_position: vec3<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    out.world_normal = skin_normals(model, vertex.normal);
#else
    var model = mesh.model;
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
    var noise = simplexNoise3(vertex.position);

    // out.color = vec4<f32>(vertex.position.x, noise, vertex.position.z, 1.0);
    out.color = vec4<f32>(0.5, noise, 0.5, 1.0);

    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position.x, noise, vertex.position.z, 1.0));
    var t = ship_position;
    out.world_position.y = out.world_position.y - 4.0 * sin(ship_position.z - out.world_position.z) / 100.;

    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}
