#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 _position;
layout(location = 1) in vec2 _texcoord;
layout(location = 0) out vec4 _debug;


void main() {
    _debug = vec4(_texcoord, 1.0, 1.0);
    gl_Position = vec4(_position, 0.0, 1.0);
}