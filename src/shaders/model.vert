#version 120

attribute vec3 position;
attribute vec3 normal;

varying vec3 v_normal;
varying vec3 v_position;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
    // TODO: Move this to the CPU side. It only needs to be done once,
    // not every time the shader is run.
    mat4 modelview = view * model;

    gl_Position = perspective * modelview * vec4(position, 1.0);
    
    vec4 p = modelview * vec4(position, 1.0);
    v_position = p.xyz / p.w;

    v_normal = mat3(modelview) * normal;
}

