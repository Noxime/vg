// shader.frag
#version 450

#ifndef FXAA_REDUCE_MIN
    #define FXAA_REDUCE_MIN   (1.0/ 128.0)
#endif
#ifndef FXAA_REDUCE_MUL
    #define FXAA_REDUCE_MUL   (1.0 / 8.0)
#endif
#ifndef FXAA_SPAN_MAX
    #define FXAA_SPAN_MAX     8.0
#endif

#ifndef BLOOM_SAMPLES
    #define BLOOM_SAMPLES     8
#endif

const float PI = 3.141592;

layout(location=0) out vec4 f_color;

layout(location=0) in vec3 v_position;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec2 v_uv0;

struct Light {
    vec3 position;
    vec3 color;
};

layout(set=0, binding=0) uniform Uniforms {
    mat4 u_view;
    mat4 u_proj;
    vec4 u_resolution;
    Light u_lights[8];
    uint u_light_count;
};

// framebuffer
layout(set=1, binding=0) uniform texture2D t_color;
layout(set=1, binding=1) uniform sampler s_color;

// skybox
layout(set=2, binding=0) uniform textureCube t_env;
layout(set=2, binding=1) uniform sampler s_env;
layout(set=2, binding=2) uniform textureCube t_ibl;


vec4 read_tex(vec2 uv) {
    vec4 fb = texture(sampler2D(t_color, s_color), uv);
    uv = uv * 2.0 - 1.0;
    vec3 view = normalize(vec3(uv * vec2(u_resolution.x / u_resolution.y, -1), -1));
    view = inverse(mat3(u_view)) * view;
    vec3 sky = texture(samplerCube(t_env, s_env), view).rgb;
    return vec4(mix(sky, fb.rgb, fb.a), 1);
}

vec3 sample_offset(vec2 offset) {
    vec2 uv = v_uv0 + offset / u_resolution.xy;
    return read_tex(uv).rgb;
}

float luma(vec3 color) {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

vec3 fxaa(vec3 color) {
    vec3 col_nw = sample_offset(vec2(-1, 1));
    vec3 col_ne = sample_offset(vec2(1, 1));
    vec3 col_sw = sample_offset(vec2(-1, -1));
    vec3 col_se = sample_offset(vec2(1, -1));
    
    float luma_nw = luma(col_nw);
    float luma_ne = luma(col_ne);
    float luma_sw = luma(col_sw);
    float luma_se = luma(col_se);
    float luma_m = luma(color);

    float luma_min = min(min(min(min(luma_nw, luma_ne), luma_sw), luma_se), luma_m);
    float luma_max = max(max(max(max(luma_nw, luma_ne), luma_sw), luma_se), luma_m);

    vec2 dir = vec2(
        -((luma_nw + luma_ne) - (luma_sw + luma_se)),
        ((luma_nw + luma_sw) - (luma_ne + luma_se))
    );

    float dir_reduce = max((luma_nw + luma_ne + luma_sw + luma_se) * 0.25 * FXAA_REDUCE_MUL, FXAA_REDUCE_MIN);
    float recp_dir_min = 1.0 / (min(abs(dir.x), abs(dir.y)) + dir_reduce);

    dir = min(vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
              max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
              dir * recp_dir_min)) / u_resolution.xy;

    vec3 rgb_a = 0.5 * (
        read_tex(v_uv0 + dir * (1.0 / 3.0 - 0.5)).rgb +
        read_tex(v_uv0 + dir * (2.0 / 3.0 - 0.5)).rgb);
    vec3 rgb_b = rgb_a * 0.5 + 0.25 * (
        read_tex(v_uv0 + dir * -0.5).rgb +
        read_tex(v_uv0 + dir * 0.5).rgb);

    float luma_b = luma(rgb_b);
    if ((luma_b < luma_min) || (luma_b > luma_max))
        color = rgb_a;
    else
        color = rgb_b;

    return color;
}

vec3 tonemap_aces(vec3 x) {
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0, 1);
}

void main() {
    vec4 fb = read_tex(v_uv0);
    vec3 color = fb.rgb;

    color = fxaa(color);
    color = tonemap_aces(color);
    // linear color
    // color = pow(color, vec3(1.0/2.2));

    f_color = vec4(color, 1.0);
}
