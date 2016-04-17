#version 140

uniform sampler2DArray tex;
uniform float tex_index;

in vec2 v_tex_coords;
out vec4 f_color;

void main() {
  f_color = texture(tex, vec3(v_tex_coords, tex_index));
}
