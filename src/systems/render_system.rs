use ecs::{ Process, System, DataHelper };

use components::LevelComponents;

use glium::{self, Surface};
use glium::index::PrimitiveType;

use std::io::Cursor;

use image::{self, GenericImage};
// use image::image::GenericImage;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}


pub struct RenderSystem {
    display: glium::Display,
    texture: glium::texture::CompressedSrgbTexture2d,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
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
                                         Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                                         Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                                         Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                                         Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] },
                                         ])
        };

        let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                                   vec![1 as u16, 2, 0, 3]);

        let program = program!(&display,
                               140 => {
                                   vertex: "
                                        #version 140

                                        uniform mat4 matrix;

                                        in vec2 position;
                                        in vec2 tex_coords;

                                        out vec2 v_tex_coords;

                                        void main() {
                                            gl_Position = matrix * vec4(position, 0.0, 1.0);
                                            v_tex_coords = tex_coords;
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
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
        }
    }
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = ();

    // make system passive, so we have to call it manually
    fn is_active(&self) -> bool { false }
}

impl Process for RenderSystem {
    fn process(&mut self, _: &mut DataHelper<LevelComponents, ()>) {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
                ],
            tex: &self.texture
        };

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }
}
