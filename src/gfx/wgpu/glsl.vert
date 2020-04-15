#version 450

const vec2 positions[6] = vec2[6](
    vec2(-1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2( 1.0,  1.0)
);

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_mat;
};

layout(location=1) out vec2 v_tex;

void main() {
    gl_Position = u_mat * vec4(positions[gl_VertexIndex], 0.0, 1.0);
    v_tex = positions[gl_VertexIndex] * vec2(1.0, -1.0) * 0.5 + 0.5;
}
