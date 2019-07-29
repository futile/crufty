use ecs::system::InteractProcess;
use ecs::{DataHelper, EntityIter, System};

use glium::index::PrimitiveType;
use glium::{self, Surface};

use std::fs::File;
use std::io::Read;

use crate::na::{Orthographic3, Vector2, Vector4};

use crate::components::LevelComponents;
use crate::components::{Facing, SpriteLayer};

use hprof;

use super::LevelServices;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldViewport {
    pub width: f32,
    pub height: f32,
}

impl WorldViewport {
    pub fn new(width: f32, height: f32) -> WorldViewport {
        WorldViewport {
            width,
            height,
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
    sprite_index_buffer: glium::IndexBuffer<u16>,
    physics_index_buffer: glium::IndexBuffer<u16>,
    sprite_program: glium::Program,
    physics_program: glium::Program,
    render_physics_debug: bool,
}

impl RenderSystem {
    pub fn new(display: glium::Display) -> RenderSystem {
        let vertex_buffer = {
            implement_vertex!(Vertex, position, tex_coords);

            glium::VertexBuffer::new(
                &display,
                &[
                    Vertex {
                        position: [0.0, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [0.0, 1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                ],
            )
            .unwrap()
        };

        let sprite_index_buffer =
            glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
                .unwrap();

        let physics_index_buffer =
            glium::IndexBuffer::new(&display, PrimitiveType::LineLoop, &[0 as u16, 1, 2, 3])
                .unwrap();

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

        let sprite_program = program!(&display,
                               140 => {
                                   vertex: &vertex_shader_code,
                                   fragment: &fragment_shader_code,
                               })
        .unwrap();

        fragment_shader_code.clear();
        File::open("assets/shaders/wireframe.frag")
            .unwrap()
            .read_to_string(&mut fragment_shader_code)
            .unwrap();

        let physics_program = program!(&display,
                                      140 => {
                                          vertex: &vertex_shader_code,
                                          fragment: &fragment_shader_code,
                                      })
        .unwrap();

        RenderSystem {
            display,
            unit_quad: vertex_buffer,
            sprite_index_buffer,
            physics_index_buffer,
            sprite_program,
            physics_program,
            render_physics_debug: false,
        }
    }

    pub fn toggle_physics_debug_render(&mut self) -> bool {
        self.render_physics_debug = !self.render_physics_debug;
        self.render_physics_debug
    }
}

impl System for RenderSystem {
    type Components = LevelComponents;
    type Services = LevelServices;
}

impl InteractProcess for RenderSystem {
    fn process(
        &mut self,
        camera_entities: EntityIter<'_, LevelComponents>,
        sprite_entities: EntityIter<'_, LevelComponents>,
        data: &mut DataHelper<LevelComponents, LevelServices>,
    ) {
        let _ = hprof::enter("rendering");

        let _s = hprof::enter("setup");

        let _g = hprof::enter("draw");
        let mut target = self.display.draw();
        drop(_g);

        let _g = hprof::enter("clear");
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
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
                let ortho_proj = Orthographic3::new(
                    0.0 - camera.world_viewport.width / 2.0,
                    0.0 + camera.world_viewport.width / 2.0,
                    0.0 - camera.world_viewport.height / 2.0,
                    0.0 + camera.world_viewport.height / 2.0,
                    -(SpriteLayer::MAX_DEPTH + 1.0), // +1.0 for debug rendering
                    0.0,
                )
                .into_inner();
                drop(_g);

                for e in &sprites {
                    let position = &data.position[*e];
                    let mut depth = 0.0f32;
                    if let Some(sprite) = &data.sprite.get(e) {
                        let scale = Vector2::new(sprite.info.width, sprite.info.height);
                        let view_pos = Vector2::new(
                            position.x.round() - (cpos.x - camera.world_viewport.width / 2.0),
                            position.y.round() - (cpos.y - camera.world_viewport.height / 2.0),
                        );
                        let texture = data
                            .services
                            .resource_store
                            .get_texture(sprite.info.texture_info);

                        let invert_tex_x = match data.facing.get(e) {
                            Some(Facing::Left) => true,
                            _ => false,
                        };

                        depth = sprite.sprite_layer.to_depth();

                        let uniforms = uniform! {
                            view_pos: *view_pos.as_ref(),
                            scale: *scale.as_ref(),
                            proj: *ortho_proj.as_ref(),
                            tex: &*texture,
                            tex_index: sprite.info.texture_info.idx,
                            invert_tex_x: invert_tex_x,
                            win_scale: *screen_size.as_ref(),
                            win_trans: *camera.screen_viewport.mins().coords.as_ref(),
                            depth: depth,
                        };

                        target
                            .draw(
                                &self.unit_quad,
                                &self.sprite_index_buffer,
                                &self.sprite_program,
                                &uniforms,
                                &glium::DrawParameters {
                                    depth: glium::Depth {
                                        test: glium::draw_parameters::DepthTest::IfLess,
                                        write: true,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                            )
                            .unwrap()
                    }

                    if self.render_physics_debug {
                        if let Some(cs) = &data.collision_shape.get(e) {
                            for aabb in
                                &[cs.aabb_x(position.as_vec()), cs.aabb_y(position.as_vec())]
                            {
                                let scale = Vector2::new(
                                    aabb.maxs().x - aabb.mins().x,
                                    aabb.maxs().y - aabb.mins().y,
                                );
                                let view_pos = Vector2::new(
                                    aabb.mins().x.round()
                                        - (cpos.x - camera.world_viewport.width / 2.0),
                                    aabb.mins().y.round()
                                        - (cpos.y - camera.world_viewport.height / 2.0),
                                );

                                use crate::components::CollisionType;

                                let color = match cs.collision_type() {
                                    CollisionType::Solid => Vector4::new(1.0f32, 0.0, 0.0, 1.0),
                                    CollisionType::Trigger => Vector4::new(0.0f32, 1.0, 0.0, 1.0),
                                };

                                let uniforms = uniform! {
                                    view_pos: *view_pos.as_ref(),
                                    scale: *scale.as_ref(),
                                    proj: *ortho_proj.as_ref(),
                                    invert_tex_x: false,
                                    win_scale: *screen_size.as_ref(),
                                    win_trans: *camera.screen_viewport.mins().coords.as_ref(),
                                    depth: SpriteLayer::MAX_DEPTH + depth/SpriteLayer::MAX_DEPTH,
                                    color: *color.as_ref(),
                                };

                                target
                                    .draw(
                                        &self.unit_quad,
                                        &self.physics_index_buffer,
                                        &self.physics_program,
                                        &uniforms,
                                        &glium::DrawParameters {
                                            depth: glium::Depth {
                                                test: glium::draw_parameters::DepthTest::IfLess,
                                                write: true,
                                                ..Default::default()
                                            },
                                            polygon_mode: glium::PolygonMode::Line,
                                            ..Default::default()
                                        },
                                    )
                                    .unwrap()
                            }
                        }
                    }
                }
            }
        }

        let _g = hprof::enter("finishing");
        target.finish().unwrap();
        drop(_g);
    }
}
