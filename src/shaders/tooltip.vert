#version 140

uniform mat3x3 transform;

in vec3 pos;
in vec2 uv;

out vec2 vert_uv;

void main() {
    gl_Position = vec4(vec3(pos.xy, 1.0) * transform - vec3(0.0, 0.0, 1.0), 1.0);
    vert_uv = uv;
}