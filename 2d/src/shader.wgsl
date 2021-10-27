// Generic struct containing the data for whatever shape we are drawing
[[block]]
struct Locals {
    color: vec4<f32>;
    props: vec4<f32>; // Roundness, Outline, Kind, Unused
    xyzw: vec4<f32>;
    uvst: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[block]]
struct Globals {
    bounds: vec4<f32>;
    resolution: vec2<u32>;
};

[[group(0), binding(1)]]
var<uniform> r_globals: Globals;

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
    [[builtin(frag_depth)]] depth: f32;
};

fn dot2(x: vec2<f32>) -> f32 {
    return dot(x, x);
}

// Bounding box functions

fn pixel_size() -> f32 {
    return 2.0 / f32(min(r_globals.resolution.x, r_globals.resolution.y));
}

fn bb_line(i: i32) -> vec2<f32> {
    let r = r_locals.props.x + r_locals.props.y + pixel_size();
    let a = r_locals.xyzw.xy;
    let b = r_locals.xyzw.zw;
    let d0 = normalize(b - a) * r; // radius-length vector along the rect
    let d90 = vec2<f32>(d0.y, -d0.x); // 90 degrees to clockwise
    switch (i) {
        case 0: { return a + d90 - d0; }
        case 1: { return a - d90 - d0; }
        case 2: { return b + d90 + d0; }
        case 3: { return b - d90 + d0; }
    }
    return vec2<f32>(0.0, 0.0);
}

fn bb_circle(i: i32) -> vec2<f32> {
    let r = r_locals.props.x + r_locals.props.y + pixel_size();
    switch (i) {
        case 0: { return r_locals.xyzw.xy + vec2<f32>(-r, -r); }
        case 1: { return r_locals.xyzw.xy + vec2<f32>(-r, r); }
        case 2: { return r_locals.xyzw.xy + vec2<f32>(r, -r); }
        case 3: { return r_locals.xyzw.xy + vec2<f32>(r, r); }
    }
    return vec2<f32>(0.0, 0.0);
}

// A rect really is more of a line with sharp corner width, so from A to B (xyzw)
// with thickness of theta (u)
fn bb_rect(i: i32) -> vec2<f32> {
    let r = r_locals.props.x + r_locals.props.y + pixel_size();
    let a = r_locals.xyzw.xy;
    let b = r_locals.xyzw.zw;

    let width = distance(a, b) / 2.0;
    let height = r_locals.uvst.x / 2.0;
    // TODO: This isn't ideal, because long but narrow rects will get lots of
    // overflow
    let d = normalize(b - a);
    let d0 = d * r; // radius-length vector along the rect
    let d90 = vec2<f32>(d.y, -d.x) * (max(width, height) + r); // 90 degrees to clockwise
    switch (i) {
        case 0: { return a + d90 - d0; }
        case 1: { return a - d90 - d0; }
        case 2: { return b + d90 + d0; }
        case 3: { return b - d90 + d0; }
    }
    return vec2<f32>(0.0, 0.0);
}

fn bb_triangle(i: i32) -> vec2<f32> {
    let r = r_locals.props.x + r_locals.props.y + pixel_size();

    let m = max(max(r_locals.xyzw.xy, r_locals.xyzw.zw), r_locals.uvst.xy);
    let n = min(min(r_locals.xyzw.xy, r_locals.xyzw.zw), r_locals.uvst.xy);

    switch (i) {
        case 0: { return vec2<f32>(n.x - r, n.y - r); }
        case 1: { return vec2<f32>(n.x - r, m.y + r); }
        case 2: { return vec2<f32>(m.x + r, n.y - r); }
        case 3: { return vec2<f32>(m.x + r, m.y + r); }
    }
    return vec2<f32>(0.0, 0.0);
}

fn bb_bezier(i: i32) -> vec2<f32> {
    let r = r_locals.props.x + r_locals.props.y + pixel_size();

    let m = max(max(r_locals.xyzw.xy, r_locals.xyzw.zw), max(r_locals.uvst.xy, r_locals.uvst.zw));
    let n = min(min(r_locals.xyzw.xy, r_locals.xyzw.zw), min(r_locals.uvst.xy, r_locals.uvst.zw));

    switch (i) {
        case 0: { return vec2<f32>(n.x - r, n.y - r); }
        case 1: { return vec2<f32>(n.x - r, m.y + r); }
        case 2: { return vec2<f32>(m.x + r, n.y - r); }
        case 3: { return vec2<f32>(m.x + r, m.y + r); }
    }
    return vec2<f32>(0.0, 0.0);
}

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var xy: vec2<f32>;

    // I should really use vertex buffers...
    switch (i32(in_vertex_index) + i32(r_locals.props.z) * 6) {
        // Line
        case 0: { xy = bb_line(0); }
        case 1: { xy = bb_line(1); }
        case 2: { xy = bb_line(2); }
        case 3: { xy = bb_line(1); }
        case 4: { xy = bb_line(2); }
        case 5: { xy = bb_line(3); }
        // Circle
        case 6: { xy = bb_circle(0); }
        case 7: { xy = bb_circle(1); }
        case 8: { xy = bb_circle(2); }
        case 9: { xy = bb_circle(1); }
        case 10: { xy = bb_circle(2); }
        case 11: { xy = bb_circle(3); }
        // Rectangle
        case 12: { xy = bb_rect(0); }
        case 13: { xy = bb_rect(1); }
        case 14: { xy = bb_rect(2); }
        case 15: { xy = bb_rect(1); }
        case 16: { xy = bb_rect(2); }
        case 17: { xy = bb_rect(3); }
        // Triangle
        case 18: { xy = bb_triangle(0); }
        case 19: { xy = bb_triangle(1); }
        case 20: { xy = bb_triangle(2); }
        case 21: { xy = bb_triangle(1); }
        case 22: { xy = bb_triangle(2); }
        case 23: { xy = bb_triangle(3); }
        // Bezier
        case 24: { xy = bb_bezier(0); }
        case 25: { xy = bb_bezier(1); }
        case 26: { xy = bb_bezier(2); }
        case 27: { xy = bb_bezier(1); }
        case 28: { xy = bb_bezier(2); }
        case 29: { xy = bb_bezier(3); }

        // Fallback with fullscreen quad, a lot of overdraw
        default: {
            switch (i32(in_vertex_index)) {
                case 0: { xy = vec2<f32>(-1.0, -1.0); }
                case 1: { xy = vec2<f32>(-1.0, 1.0); }
                case 2: { xy = vec2<f32>(1.0, -1.0); }
                case 3: { xy = vec2<f32>(-1.0, 1.0); }
                case 4: { xy = vec2<f32>(1.0, -1.0); }
                case 5: { xy = vec2<f32>(1.0, 1.0); }
            }
        }
    }

    //xy = mix(r_globals.bounds.xy, r_globals.bounds.zw, xy * 0.5 + 0.5);

    out.position = vec4<f32>(mix(1.0 / r_globals.bounds.xy, 1.0 / r_globals.bounds.zw, xy * 0.5 + 0.5), 0.0, 1.0);
    out.tex_coord = xy;

    return out;
}

// Distance field functions

fn sdf_line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn sdf_circle(p: vec2<f32>, a: vec2<f32>) -> f32 {
    return distance(p, a);
}

fn sdf_rect(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, th: f32) -> f32 {
    let l = length(b - a);
    let d = (b - a) / l;
    var q = (p - (a + b) * 0.5);
    q = mat2x2<f32>(vec2<f32>(d.x, -d.y), vec2<f32>(d.y, d.x)) * q;
    q = abs(q) - vec2<f32>(l, th) * 0.5;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0); 
}

fn sdf_triangle(p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    let e0 = p1-p0; let e1 = p2-p1; let e2 = p0-p2;
    let v0 = p -p0; let v1 = p -p1; let v2 = p -p2;
    let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0 );
    let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0 );
    let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0 );
    let s = sign(e0.x * e2.y - e0.y * e2.x);
    let d = min(min(vec2<f32>(dot(pq0, pq0), s * (v0.x * e0.y-v0.y * e0.x)),
                    vec2<f32>(dot(pq1, pq1), s * (v1.x * e1.y-v1.y * e1.x))),
                    vec2<f32>(dot(pq2, pq2), s * (v2.x * e2.y-v2.y * e2.x)));
    return -sqrt(d.x) * sign(d.y);
}

fn sdf_bezier(pos: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> f32 {
    let kNum = 24;
    var res = 1e38;
    var a = p0;

    var i = 1;
    loop {
        if (i == kNum) { break; }

        let t = f32(i) / f32(kNum - 1);
        let s = 1.0 - t;
        let b = p0 * s * s * s + 
                p1 * 3.0 * s * s * t + 
                p2 * 3.0 * s * t * t + 
                p3 * t * t * t;
        let d = sdf_line(pos, a, b);
        res = min(res, d);
        a = b;

        i = i + 1;
    }

    return res;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let uv = in.tex_coord;
    let rounding = r_locals.props.x;
    let outline = r_locals.props.y;
    let kind = r_locals.props.z;

    // Calculate distance to shape
    var t: f32;
    switch (i32(kind)) {
        case 0: { t = sdf_line(uv, r_locals.xyzw.xy, r_locals.xyzw.zw); }
        case 1: { t = sdf_circle(uv, r_locals.xyzw.xy); }
        case 2: { t = sdf_rect(uv, r_locals.xyzw.xy, r_locals.xyzw.zw, r_locals.uvst.x); }
        case 3: { t = sdf_triangle(uv, r_locals.xyzw.xy, r_locals.xyzw.zw, r_locals.uvst.xy); }
        case 4: { t = sdf_bezier(uv, r_locals.xyzw.xy, r_locals.xyzw.zw, r_locals.uvst.xy, r_locals.uvst.zw); }
    }

    // Apply rounding/width
    t = t - r_locals.props.x;

    // Make it hollow
    if (outline > 0.0) {
        t = abs(t) - outline;
    }

    // Analytic anti-aliasing
    let w = fwidth(t) * 0.5;
    let blend = vec4<f32>(1.0, 1.0, 1.0, 1.0 - smoothStep(-w, w, t));

    out.color = r_locals.color * blend;
    out.depth = t;

    return out;
}