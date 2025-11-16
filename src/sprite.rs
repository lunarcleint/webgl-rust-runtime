#![allow(unused)]
use crate::{
    assets::{self, Image},
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
        /* Load shader or use default shader */
        let program = shader.unwrap_or_else(|| {
            render::with_renderer(|renderer| {
                renderer.base_program.as_ref().unwrap().as_ref().clone()
            })
        });

        /* Load image from assets */
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
            let texture = &image.borrow().webl_gl_texture;

            renderer.use_program(&self.shader);
            renderer.use_texture(texture);

            let camera = self.camera.borrow();
            let vertices = camera.transform_tris(self);

            renderer
                .quads_buffer
                .upload_vertices(&renderer.context, &vertices);
            renderer
                .quads_buffer
                .upload_uvs(&renderer.context, &BASE_QUAD_UVS);
            renderer
                .quads_buffer
                .upload_indices(&renderer.context, &BASE_QUAD_INDICES);

            renderer.draw_triangles(BASE_QUAD_INDICES.len() as i32);
        }
    }
}
