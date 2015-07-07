#version 140

uniform vec2 view_pos;
uniform vec2 scale;
uniform mat4 proj;

in vec2 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
  v_tex_coords = tex_coords;

  vec4 pos = proj * vec4(scale * position + view_pos,  1.0, 1.0);
  pos.xy -= vec2(1.0, 1.0);

  gl_Position = pos;
}
