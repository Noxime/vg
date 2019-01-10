#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 _position;
layout(location = 1) in vec2 _texcoord;
layout(location = 0) out vec2 v_texcoord;

layout(binding = 2) uniform UniformBlock {
    mat4 projection;
} uniform_block;


void main() {
    v_texcoord = _texcoord;
    gl_Position = uniform_block.projection * vec4(_position, 0.0, 1.0);
}