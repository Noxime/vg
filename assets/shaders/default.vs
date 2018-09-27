#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 _position;
layout(location = 0) out vec4 _debug;

void main() {
    _debug = vec4(_position, 1.0);
    gl_Position = vec4(_position, 1.0);
}