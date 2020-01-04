#version 140

uniform bool light;

out vec4 out_color;
in vec3 vert_color;
in vec3 vert_normal;

void main() {
    float shade = (dot(normalize(vert_normal), vec3(0.0, 1.0, 0.0)) + 1.0) / 2.0;
    if (!light) {
        shade = 1.0;
    }
    out_color = vec4(vert_color * shade, 1.0);
}