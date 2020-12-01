#version 450

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

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 a_uv0;

layout(location=0) out vec3 v_position;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec2 v_uv0;

void main() {
    mat4 u_model = mat4(1);
    mat4 vp = u_proj * u_view;
    mat3 normal_matrix = mat3(transpose(inverse(u_model)));

    vec4 model_space = u_model * vec4(a_position, 1.0);

    gl_Position = vp * model_space;
    v_position = model_space.xyz;
    v_normal = normal_matrix * a_normal;
    v_uv0 = a_uv0;
}
