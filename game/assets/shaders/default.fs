#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 v_texcoord;
layout(location = 0) out vec4 _out_color;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

void main() {
    // _out_color = vec4(0.2, 0.5, 0.7, 1.0);
    _out_color = texture(sampler2D(u_texture, u_sampler), v_texcoord);
}