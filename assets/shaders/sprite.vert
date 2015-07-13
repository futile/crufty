#version 140

uniform vec2 view_pos;
uniform vec2 scale;
uniform mat4 proj;
uniform vec2 win_scale;
uniform vec2 win_trans;

in vec2 position;
in vec2 tex_coords;

out vec2 v_tex_coords;

void main() {
  v_tex_coords = tex_coords;

  // world -> [0.0, 2.0]
  vec4 pos = proj * vec4(scale * position + view_pos,  1.0, 1.0);

  // -> window coords
  pos.xy *= win_scale / 2.0;
  pos.xy += win_trans;

  gl_Position = pos;
}
