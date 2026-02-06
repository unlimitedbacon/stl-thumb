#version 150

in vec3 position;
in vec3 normal;

out vec3 v_normal;
out vec3 v_position;

uniform mat4 perspective;
uniform mat4 modelview;

void main() {
    gl_Position = perspective * modelview * vec4(position, 1.0);
    
    vec4 p = modelview * vec4(position, 1.0);
    v_position = p.xyz / p.w;

    v_normal = mat3(modelview) * normal;
}
