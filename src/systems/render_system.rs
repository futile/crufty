use ecs::{ System, DataHelper, EntityIter };
use ecs::system::EntityProcess;

use components::LevelComponents;

use glium::{self, Surface};
use glium::index::PrimitiveType;

use std::io::Cursor;

use image::{self, GenericImage};

use na::{Vec2, OrthoMat3};

#[derive(Copy, Clone, PartialEq, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct WorldViewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl WorldViewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> WorldViewport {
        WorldViewport {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }

    pub fn empty() -> WorldViewport {
        WorldViewport::new(0.0, 0.0, 0.0, 0.0)
    }
}

pub struct RenderSystem {
    display: glium::Display,
    texture: glium::texture::CompressedSrgbTexture2d,
    unit_quad: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,

    pub world_viewport: WorldViewport,
}

impl RenderSystem {
    pub fn new(display: glium::Display) -> RenderSystem {
        let mut image = image::load(Cursor::new(&include_bytes!("../../assets/test1.png")[..]), image::PNG).unwrap();

        let image = image.sub_image(0, 0, 32, 32).to_image();
        let texture = glium::texture::CompressedSrgbTexture2d::new(&display, image);

        let vertex_buffer = {
            implement_vertex!(Vertex, position, tex_coords);

            glium::VertexBuffer::new(&display,
                                     vec![
                                         Vertex { position: [ 0.0,  0.0], tex_coords: [0.0, 0.0] },
                                         Vertex { position: [ 0.0,  1.0], tex_coords: [0.0, 1.0] },
                                         Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                                         Vertex { position: [ 1.0,  0.0], tex_coords: [1.0, 0.0] },
                                         ])
        };

        let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                                   vec![1 as u16, 2, 0, 3]);

        let program = program!(&display,
                               140 => {
                                   vertex: "
                                        #version 140

                                        uniform vec2 view_pos;
                                        uniform vec2 scale;
                                        uniform mat4 proj;

                                        in vec2 position;
                                        in vec2 tex_coords;

                                        out vec2 v_tex_coords;

                                        void main() {
                                            vec4 pos = proj * vec4(scale * position + view_pos,  1.0, 1.0);
                                            pos.xy -= vec2(1.0, 1.0);
                                            v_tex_coords = tex_coords;
                                            gl_Position = pos;
                                        }
                                    ",

                                   fragment: "
                                        #version 140

                                        uniform sampler2D tex;
                                        in vec2 v_tex_coords;
                                        out vec4 f_color;

                                        void main() {
                                            f_color = texture(tex, v_tex_coords);
                                        }
                                    "
                               }).unwrap();

        RenderSystem {
            display: display,
            texture: texture,
            unit_quad: vertex_buffer,
            index_buffer: index_buffer,
            program: program,

            world_viewport: WorldViewport::empty(),
        }
    }
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = ();

    // make system passive, so we have to call it manually
    fn is_active(&self) -> bool { false }
}

impl EntityProcess for RenderSystem {
    fn process(&mut self, entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, ()>) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        let ortho_proj = OrthoMat3::new(self.world_viewport.width, self.world_viewport.height, 0.0, -2.0);

        for e in entities {
            let position = data.position[e];
            let sprite_info = data.sprite_info[e];

            let scale = Vec2::new(sprite_info.width, sprite_info.height);
            let view_pos = Vec2::new(position.x - self.world_viewport.x, position.y - self.world_viewport.y);

            let uniforms = uniform! {
                view_pos: view_pos,
                scale: scale,
                proj: ortho_proj.to_mat(),
                tex: &self.texture
            };

            target.draw(&self.unit_quad, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        }

        target.finish().unwrap();
    }
}
