#ifndef MATERIAL_H
#define MATERIAL_H

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

#endif