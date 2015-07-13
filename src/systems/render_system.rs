use ecs::{ System, DataHelper, EntityIter };
use ecs::system::InteractProcess;

use glium::{self, Surface};
use glium::index::PrimitiveType;

use std::io::Read;
use std::fs::File;

use image::{self, GenericImage};

use na::{Vec2, OrthoMat3};

use components::LevelComponents;

use hprof;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct WorldViewport {
    pub width: f32,
    pub height: f32,
}

impl WorldViewport {
    pub fn new(width: f32, height: f32) -> WorldViewport {
        WorldViewport {
            width: width,
            height: height,
        }
    }

    pub fn empty() -> WorldViewport {
        WorldViewport::new(0.0, 0.0)
    }
}

pub struct RenderSystem {
    display: glium::Display,
    texture: glium::texture::CompressedSrgbTexture2d,
    unit_quad: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
}

impl RenderSystem {
    pub fn new(display: glium::Display) -> RenderSystem {
        let mut image = image::open("../../assets/test1.png").unwrap();

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

        let mut vertex_shader_code = String::new();
        File::open("../../assets/shaders/sprite.vert")
            .unwrap()
            .read_to_string(&mut vertex_shader_code)
            .unwrap();

        let mut fragment_shader_code = String::new();
        File::open("../../assets/shaders/sprite.frag")
            .unwrap()
            .read_to_string(&mut fragment_shader_code)
            .unwrap();

        let program = program!(&display,
                               140 => {
                                   vertex: &vertex_shader_code,
                                   fragment: &fragment_shader_code,
                               }).unwrap();

        RenderSystem {
            display: display,
            texture: texture,
            unit_quad: vertex_buffer,
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

impl InteractProcess for RenderSystem {
    fn process(&self, camera_entities: EntityIter<LevelComponents>, sprite_entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, ()>) {
        let _ = hprof::enter("rendering");

        let _s = hprof::enter("setup");

        let _g = hprof::enter("draw");
        let mut target = self.display.draw();
        drop(_g);

        let _g = hprof::enter("clear");
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        drop(_g);

        drop(_s);

        {
            let _g = hprof::enter("drawing");

            let sprites: Vec<_> = sprite_entities.collect();

            for ce in camera_entities {
                let ref camera = data.camera[ce];
                let cpos = data.position[ce];

                let _g = hprof::enter("ortho");
                let ortho_proj = OrthoMat3::new(camera.world_viewport.width, camera.world_viewport.height, 0.0, -2.0).to_mat();
                drop(_g);

                for e in &sprites {
                    let position = data.position[*e];
                    let sprite_info = data.sprite_info[*e];

                    let scale = Vec2::new(sprite_info.width, sprite_info.height);
                    let view_pos = Vec2::new(position.x - cpos.x, position.y - cpos.y);

                    let uniforms = uniform! {
                        view_pos: view_pos,
                        scale: scale,
                        proj: ortho_proj,
                        tex: &self.texture
                    };

                    target.draw(&self.unit_quad, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap()
                }
            }
        }

        let _g = hprof::enter("finishing");
        target.finish().unwrap();
        drop(_g);
    }
}
