
// #include "uniforms.h"

#ifndef MATERIAL_H
#define MATERIAL_H

const float PI = 3.141592;

layout(location=0) out vec4 f_color;

layout(location=0) in vec3 v_position;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec2 v_uv0;

struct Material {
    vec4 color;
    float reflectance;
    float roughness;
    float metallic;
    float clear_coat;
    float clear_coat_roughness;
    vec4 emission;
};

layout(set=1, binding=0) uniform texture2D t_color;
layout(set=1, binding=1) uniform sampler s_color;
layout(set=1, binding=2) uniform texture2D t_reflectance;
layout(set=1, binding=3) uniform sampler s_reflectance;
layout(set=1, binding=4) uniform texture2D t_roughness;
layout(set=1, binding=5) uniform sampler s_roughness;
layout(set=1, binding=6) uniform texture2D t_metallic;
layout(set=1, binding=7) uniform sampler s_metallic;
layout(set=1, binding=8) uniform texture2D t_clear_coat;
layout(set=1, binding=9) uniform sampler s_clear_coat;
layout(set=1, binding=10) uniform texture2D t_clear_coat_roughness;
layout(set=1, binding=11) uniform sampler s_clear_coat_roughness;
layout(set=1, binding=12) uniform texture2D t_emission;
layout(set=1, binding=13) uniform sampler s_emission;

layout(set=2, binding=0) uniform textureCube t_env;
layout(set=2, binding=1) uniform sampler s_env;

Material load_mat() {
    Material mat = Material(
        texture(sampler2D(t_color, s_color), v_uv0),
        texture(sampler2D(t_reflectance, s_reflectance), v_uv0).r,
        texture(sampler2D(t_roughness, s_roughness), v_uv0).r,
        texture(sampler2D(t_metallic, s_metallic), v_uv0).r,
        texture(sampler2D(t_clear_coat, s_clear_coat), v_uv0).r,
        texture(sampler2D(t_clear_coat_roughness, s_clear_coat_roughness), v_uv0).r,
        texture(sampler2D(t_emission, s_emission), v_uv0)
    );
    return mat;
} 

#endif