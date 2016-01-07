use ecs::{ System, DataHelper, EntityIter };
use ecs::system::InteractProcess;

use glium::{self, Surface};
use glium::index::PrimitiveType;

use std::fs::File;
use std::io::Read;

use image::{GenericImage};

use na::{Vec2, OrthoMat3};

use components::LevelComponents;

use hprof;

use super::LevelServices;

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

    #[allow(dead_code)]
    pub fn new_empty() -> WorldViewport {
        WorldViewport::new(0.0, 0.0)
    }
}

pub struct RenderSystem {
    display: glium::Display,
    unit_quad: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
}

impl RenderSystem {
    pub fn new(display: glium::Display) -> RenderSystem {
        let vertex_buffer = {
            implement_vertex!(Vertex, position, tex_coords);

            glium::VertexBuffer::new(&display,
                                     &[
                                         Vertex { position: [ 0.0,  0.0], tex_coords: [0.0, 0.0] },
                                         Vertex { position: [ 0.0,  1.0], tex_coords: [0.0, 1.0] },
                                         Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                                         Vertex { position: [ 1.0,  0.0], tex_coords: [1.0, 0.0] },
                                         ]).unwrap()
        };

        let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                                   &[1 as u16, 2, 0, 3]).unwrap();

        let mut vertex_shader_code = String::new();
        File::open("assets/shaders/sprite.vert")
            .unwrap()
            .read_to_string(&mut vertex_shader_code)
            .unwrap();

        let mut fragment_shader_code = String::new();
        File::open("assets/shaders/sprite.frag")
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
            unit_quad: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
        }
    }
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = LevelServices;

    // make system passive, so we have to call it manually
    fn is_active(&self) -> bool { false }
}

impl InteractProcess for RenderSystem {
    fn process(&mut self, camera_entities: EntityIter<LevelComponents>, sprite_entities: EntityIter<LevelComponents>, data: &mut DataHelper<LevelComponents, LevelServices>) {
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
                let camera = &data.camera[ce];
                let cpos = data.position[ce];
                let screen_size = camera.screen_viewport.half_extents() * 2.0;

                let _g = hprof::enter("ortho");
                let ortho_proj = OrthoMat3::new(camera.world_viewport.width, camera.world_viewport.height, 0.0, -2.0).to_mat();
                drop(_g);

                for e in &sprites {
                    let position = &data.position[*e];
                    let sprite_info = &data.sprite_info[*e];

                    let scale = Vec2::new(sprite_info.width, sprite_info.height);
                    let view_pos = Vec2::new(position.x.round() - (cpos.x - camera.world_viewport.width / 2.0), position.y.round() - (cpos.y - camera.world_viewport.height / 2.0));

                    let texture = data.services.texture_store.get_texture(&sprite_info.texture_info);

                    let uniforms = uniform! {
                        view_pos: view_pos.as_ref().clone(),
                        scale: scale.as_ref().clone(),
                        proj: ortho_proj.as_ref().clone(),
                        tex: texture,
                        tex_index: sprite_info.texture_info.idx,
                        win_scale: screen_size.as_ref().clone(),
                        win_trans: camera.screen_viewport.mins().to_vec().as_ref().clone(),
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
