#![allow(unused)]
use crate::{
    assets::{self, Image},
    camera::DrawCall,
    log,
    render::{BASE_QUAD_INDICES, BASE_QUAD_UVS, BASE_QUAD_VERTS},
};
use std::{cell::RefCell, rc::Rc};

use crate::{
    camera::{self, Camera},
    console_log,
    object::Object,
    render::{self, Renderer},
};
use web_sys::{HtmlImageElement, WebGlProgram, WebGlTexture};

pub struct Sprite {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub scalex: f32,
    pub scaley: f32,

    pub rotation: f32,

    pub camera: Rc<RefCell<Camera>>,
    pub image: Option<Rc<RefCell<Image>>>,
    pub shader: WebGlProgram,
}

impl Sprite {
    pub async fn new(
        x: f32,
        y: f32,
        camera: Rc<RefCell<Camera>>,
        image: &str,
        shader: Option<WebGlProgram>,
    ) -> Sprite {
        let program = shader.unwrap_or_else(|| {
            render::with_renderer(|renderer| {
                renderer.base_program.as_ref().unwrap().as_ref().clone()
            })
        });

        let image_ref = assets::Assets::load_image(image).await;

        let image_binding = image_ref.clone();
        let image_pointer = &image_binding.as_ref();

        let mut width = 0.0;
        let mut height = 0.0;

        if let Some(pointer) = image_pointer {
            let borrowed = pointer.borrow();

            let html_image = &borrowed.html_image;
            let webl_gl_texture = &borrowed.webl_gl_texture;

            width = html_image.width() as f32;
            height = html_image.height() as f32;

            render::with_renderer(|renderer| {
                renderer.set_texture_filtering(webl_gl_texture, true);

                renderer.use_program(&program);

                renderer.bind_vert_attribs(&renderer.quads_buffer, &program);
                renderer.bind_frag_uniforms(&program, webl_gl_texture);
            });
        }

        Sprite {
            x,
            y,

            width,
            height,

            scalex: 1.0,
            scaley: 1.0,

            rotation: 0.0,

            camera,
            image: image_ref,
            shader: program,
        }
    }
}

impl Object for Sprite {
    fn update(&mut self, _delta_time: f32) {}

    fn draw(&self, renderer: &render::Renderer) {
        if let Some(ref image) = self.image {
            let mut camera = self.camera.borrow_mut();
            let vertices = camera.transform_tris(self);
            let top = camera.draws.last_mut();

            let texture = &image.borrow().webl_gl_texture;

            /* Batch draws that use the same texture and program */
            if let Some(draw) = top {
                if draw.program == self.shader && draw.texture == *texture {
                    draw.vertices.append(&mut vertices.clone());

                    draw.count += 1;
                    return;
                }
            }

            let draw_call = DrawCall {
                texture: texture.clone(),
                program: self.shader.clone(),
                vertices,
                count: 1,
            };

            camera.draws.push(draw_call);
        }
    }
}
