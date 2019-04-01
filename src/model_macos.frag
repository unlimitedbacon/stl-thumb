#version 330

out vec3 v_normal;
out vec3 v_position;

uniform vec3 u_light;

uniform vec3 ambient_color;
uniform vec3 diffuse_color;
uniform vec3 specular_color;

out vec4 fragColor;

void main() {
    float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);

    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(u_light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

    // Alternative specular method
    // vec3 R = reflect( normalize(-u_light), normalize(v_normal) );
    // float cosAlpha = clamp( dot(camera_dir,R), 0, 1 );
    // float specular = pow( cosAlpha, 4.0 );

    fragColor = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}

