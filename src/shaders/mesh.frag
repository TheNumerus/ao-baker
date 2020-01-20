#version 140

uniform bool light;
uniform bool ao;

out vec4 out_color;
in vec3 vert_color;
in vec3 vert_normal;

void main() {
    float shade = (dot(normalize(vert_normal), vec3(0.0, 1.0, 0.0)) + 1.0) / 2.0;
    if (!light) {
        shade = 1.0;
    }
    if (!ao) {
        out_color = vec4(vec3(1.0) * shade, 1.0);
    } else {
        out_color = vec4(vert_color * shade, 1.0);
    }
}