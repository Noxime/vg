#version 450

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
layout(set=2, binding=2) uniform textureCube t_ibl;

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

float dist_ggx(vec3 n, vec3 h, float a) {
    float a2     = a*a;
    float n_dot_h  = max(dot(n, h), 0.0);
    float n_dot_h2 = n_dot_h*n_dot_h;
	
    float nom    = a2;
    float denom  = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denom        = PI * denom * denom;
	
    return nom / denom;
}

float geometry_shlick_ggx(float ndotv, float k)
{
    float nom   = ndotv;
    float denom = ndotv * (1.0 - k) + k;
	
    return nom / denom;
}
  
float geometry_smith_ggx(vec3 n, vec3 v, vec3 l, float k)
{
    float ndotv = max(dot(n, v), 0.0);
    float ndotl = max(dot(n, l), 0.0);
    float ggx1 = geometry_shlick_ggx(ndotv, k);
    float ggx2 = geometry_shlick_ggx(ndotl, k);
	
    return ggx1 * ggx2;
}

vec3 fresnel_shlick(float cosine, vec3 f0)
{
  return f0 + (1-f0) * pow(1 - cosine, 5);
}

void main() {
    vec3 view_pos = (inverse(u_view) * vec4(0, 0, 0, 1)).xyz;
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(view_pos - v_position);
    vec3 reflect_dir = reflect(-view_dir, normal);

    Material mat = load_mat();

    // thanks LearnOpenGL
    vec3 f0 = vec3(0.04);
    f0 = mix(f0, mat.color.rgb, mat.metallic);

    // vec3 ibl_radiance = texture(samplerCube(t_env, s_env), reflect_dir).rgb;
    // vec3 ibl_half_dir = normalize(view_dir + reflect_dir);
    // float ibl_ndf = dist_ggx(normal, reflect_dir, mat.roughness);
    // float ibl_g = geometry_smith_ggx(normal, view_dir, reflect_dir, mat.roughness);
    // vec3 ibl_f = fresnel_shlick(max(dot(ibl_half_dir, reflect_dir), 0), f0);

    // vec3 ibl_ks = ibl_f;
    // vec3 ibl_kd = vec3(1) - ibl_ks;
    // ibl_kd *= 1.0 - mat.metallic;

    // vec3 ibl_numerator = ibl_ndf * ibl_g * ibl_f;
    // float ibl_denominator = 4.0 * max(dot(normal, view_dir), 0) * max(dot(normal, reflect_dir), 0);
    // vec3 ibl_specular = ibl_numerator / max(ibl_denominator, 0.001) * mat.reflectance;

    // float ibl_ndotl = max(dot(normal, reflect_dir), 0);
    // vec3 ibl = (ibl_kd * mat.color.rgb / PI + ibl_specular) * ibl_radiance * ibl_ndotl;

    vec3 lo = vec3(0);

    for(uint i = 0; i < u_light_count; i++) {
        vec3 light_dir = normalize(u_lights[i].position - v_position);
        vec3 half_dir = normalize(view_dir + light_dir);
        float distance = distance(u_lights[i].position, v_position);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = u_lights[i].color * attenuation;

        float ndf = dist_ggx(normal, light_dir, mat.roughness);
        float g = geometry_smith_ggx(normal, view_dir, light_dir, mat.roughness);
        vec3 f = fresnel_shlick(max(dot(half_dir, view_dir), 0), f0); 

        vec3 ks = f;
        vec3 kd = vec3(1) - ks;
        kd *= 1.0 - mat.metallic;

        vec3 numerator = ndf * g * f;
        float denominator = 4.0 * max(dot(normal, view_dir), 0) * max(dot(normal, light_dir), 0);
        vec3 specular = numerator / max(denominator, 0.001) * mat.reflectance;

        float ndotl = max(dot(normal, light_dir), 0);
        lo += (kd * mat.color.rgb / PI + specular) * radiance * ndotl;
    }

    vec3 ambient = texture(samplerCube(t_ibl, s_env), normal).rgb * mat.color.rgb;
    vec3 color = ambient + lo;

    color = color / (color + vec3(1));

    color += mat.emission.rgb;
    // linear color
    // color = pow(color, vec3(1.0/2.2));

    f_color = vec4(color, mat.color.a);
}
