#version 150

in vec2 position;
in vec2 i_tex_coords;

out vec2 v_tex_coords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_tex_coords = i_tex_coords;
}
