#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 _debug;
layout(location = 0) out vec4 _out_color;

void main() {
    _out_color = _debug;
    _out_color = vec4(0.2, 0.5, 0.7, 1.0);
}