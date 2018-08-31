uniform vec2 u_resolution;

attribute vec3 a_position;
attribute vec2 a_texcoord;

varying vec2 v_texcoord;

void main() {
    v_texcoord = a_texcoord;
    gl_Position = vec4(a_position, 1);

    // fix aspect ratio, change this if you dont want to be controlled by window height
    gl_Position.x /= u_resolution.x / u_resolution.y;
}