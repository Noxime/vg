#version 100
precision mediump float;

uniform sampler2D tex;

varying vec2 v_tex_coords;

void main() {
    gl_FragColor = texture2D(tex, vec2(v_tex_coords.s, 1.0 - v_tex_coords.t));
}