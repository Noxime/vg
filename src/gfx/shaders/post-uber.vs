// post-uber.vs
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;
layout(location=2) in vec2 a_uv0;

layout(location=0) out vec3 v_position;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec2 v_uv0;

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

void main() {
    gl_Position = vec4(a_position, 1.0);
    v_position = a_position;
    v_normal = a_normal;
    v_uv0 = a_uv0;
}
