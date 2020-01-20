#version 140

uniform sampler2D font_texture;

in vec2 vert_uv;
out vec4 out_color;

void main() {
    vec4 image = texture(font_texture, vert_uv);
    out_color = vec4(1.0, 1.0, 1.0, image.a);
}