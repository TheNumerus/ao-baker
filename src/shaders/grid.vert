#version 140

uniform mat4x4 view;
uniform mat4x4 world;

in vec3 pos;
in vec3 normal;
in vec3 color;

void main() {
    gl_Position = view * world * vec4(pos, 1.0);
}