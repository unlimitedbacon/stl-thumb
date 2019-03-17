#version 120

attribute vec3 position;
attribute vec3 normal;

varying vec3 v_normal;
varying vec3 v_position;

uniform mat4 perspective;
//uniform mat4 view;
//uniform mat4 model;
uniform mat4 modelview;

void main() {
    // These never change, so they can be computed CPU side.
    //mat4 modelview = view * model;

    gl_Position = perspective * modelview * vec4(position, 1.0);
    
    vec4 p = modelview * vec4(position, 1.0);
    v_position = p.xyz / p.w;

    v_normal = mat3(modelview) * normal;
}

