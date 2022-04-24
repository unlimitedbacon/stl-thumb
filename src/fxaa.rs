/*
 * Example taken from
 * https://github.com/glium/glium/blob/master/examples/fxaa.rs
 * Apache License 2.0
*/

use glium::backend::Context;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::{self, Surface};

use std::cell::RefCell;
use std::rc::Rc;

pub struct FxaaSystem {
    context: Rc<Context>,
    vertex_buffer: glium::VertexBuffer<SpriteVertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
    target_color: RefCell<Option<glium::texture::Texture2d>>,
    target_depth: RefCell<Option<glium::framebuffer::DepthRenderBuffer>>,
}

#[derive(Copy, Clone)]
struct SpriteVertex {
    position: [f32; 2],
    i_tex_coords: [f32; 2],
}

implement_vertex!(SpriteVertex, position, i_tex_coords);

impl FxaaSystem {
    pub fn new<F: ?Sized>(facade: &F) -> FxaaSystem
    where
        F: Facade,
    {
        FxaaSystem {
            context: facade.get_context().clone(),

            vertex_buffer: glium::VertexBuffer::new(
                facade,
                &[
                    SpriteVertex {
                        position: [-1.0, -1.0],
                        i_tex_coords: [0.0, 0.0],
                    },
                    SpriteVertex {
                        position: [-1.0, 1.0],
                        i_tex_coords: [0.0, 1.0],
                    },
                    SpriteVertex {
                        position: [1.0, 1.0],
                        i_tex_coords: [1.0, 1.0],
                    },
                    SpriteVertex {
                        position: [1.0, -1.0],
                        i_tex_coords: [1.0, 0.0],
                    },
                ],
            )
            .unwrap(),

            index_buffer: glium::index::IndexBuffer::new(
                facade,
                glium::index::PrimitiveType::TriangleStrip,
                &[1 as u16, 2, 0, 3],
            )
            .unwrap(),

            program: program!(facade,
                100 => {
                    vertex: include_str!("shaders/fxaa.vert"),
                    fragment: include_str!("shaders/fxaa.frag"),
                }
            )
            .unwrap(),

            target_color: RefCell::new(None),
            target_depth: RefCell::new(None),
        }
    }
}

pub fn draw<T, F, R>(system: &FxaaSystem, target: &mut T, enabled: bool, mut draw: F) -> R
where
    T: Surface,
    F: FnMut(&mut SimpleFrameBuffer) -> R,
{
    let target_dimensions = target.get_dimensions();

    let mut target_color = system.target_color.borrow_mut();
    let mut target_depth = system.target_depth.borrow_mut();

    {
        let clear = if let &Some(ref tex) = &*target_color {
            tex.get_width() != target_dimensions.0
                || tex.get_height().unwrap() != target_dimensions.1
        } else {
            false
        };
        if clear {
            *target_color = None;
        }
    }

    {
        let clear = if let &Some(ref tex) = &*target_depth {
            tex.get_dimensions() != target_dimensions
        } else {
            false
        };
        if clear {
            *target_depth = None;
        }
    }

    if target_color.is_none() {
        let texture = glium::texture::Texture2d::empty(
            &system.context,
            target_dimensions.0 as u32,
            target_dimensions.1 as u32,
        )
        .unwrap();
        *target_color = Some(texture);
    }
    let target_color = target_color.as_ref().unwrap();

    if target_depth.is_none() {
        let texture = glium::framebuffer::DepthRenderBuffer::new(
            &system.context,
            glium::texture::DepthFormat::I24,
            target_dimensions.0 as u32,
            target_dimensions.1 as u32,
        )
        .unwrap();
        *target_depth = Some(texture);
    }
    let target_depth = target_depth.as_ref().unwrap();

    let output = draw(
        &mut SimpleFrameBuffer::with_depth_buffer(&system.context, target_color, target_depth)
            .unwrap(),
    );

    let uniforms = uniform! {
        tex: &*target_color,
        enabled: if enabled { 1i32 } else { 0i32 },
        resolution: (target_dimensions.0 as f32, target_dimensions.1 as f32)
    };

    target
        .draw(
            &system.vertex_buffer,
            &system.index_buffer,
            &system.program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();

    output
}
