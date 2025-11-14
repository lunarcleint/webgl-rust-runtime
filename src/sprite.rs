#![allow(unused)]
use crate::log;
use std::{cell::RefCell, rc::Rc};

use web_sys::{HtmlImageElement, WebGlProgram, WebGlTexture};
use crate::{camera::{self, Camera}, console_log, gl, object::Object, render::{self, RenderState}};

pub struct Sprite {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub scalex: f32,
    pub scaley: f32,

    pub rotation: f32,

    pub camera: Rc<RefCell<Camera>>,

    vertices: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u16>,

    texture: WebGlTexture,
    program: WebGlProgram
}

impl Sprite {
    pub fn new(x: f32, y: f32, camera: Rc<RefCell<Camera>>, image: Option<HtmlImageElement>, render: &RenderState, program: Option<WebGlProgram>) -> Sprite {
        let program = program.unwrap_or(render::create_program(render, None, None).unwrap());
        let texture = match (image) {
            Some(ref html_image) => gl::load_texture_image(&render.context, &html_image).unwrap(),
            None => gl::load_texture_empty(&render.context).unwrap()
        };
        gl::set_texture_filtering(&render.context, &texture, true);

        let vertices = gl::BASE_QUAD_VERTS;
        let uvs = gl::BASE_QUAD_UVS;
        let indices = gl::BASE_QUAD_INDICES;

        render::use_program(render, &program);

        render::upload_vertices(render, &vertices);
        render::upload_uvs(render, &uvs);
        render::upload_indices(render, &indices);

        render::bind_vert_attribs(render, &program);
        render::bind_frag_uniforms(render, &program, &texture);

        Sprite {
            x,
            y,

            width: match (image) {
                Some(ref html_image) => html_image.width() as f32,
                None => gl::BASE_TEXTURE_WIDTH as f32
            },
            height: match (image) {
                Some(ref html_image) => html_image.height() as f32,
                None => gl::BASE_TEXTURE_HEIGHT as f32
            },

            scalex: 1.0,
            scaley: 1.0,

            rotation: 0.0,

            camera,

            vertices: vertices.to_vec(),
            uvs: uvs.to_vec(),
            indices: indices.to_vec(),

            texture,
            program
        }
    }
}

impl Object for Sprite {
    fn update(&mut self, _delta_time: f32) {}

    fn draw(&mut self, render: &render::RenderState) {
        render::use_program(render, &self.program);
        render::use_texture(render, &self.texture);
        
        let camera = self.camera.borrow();
        let new_vertices = camera::transform_tris(self, &camera);

        if new_vertices != self.vertices {
            self.vertices = new_vertices;
        }

        render::upload_vertices(render, &self.vertices);
        render::upload_uvs(render, &self.uvs);
        render::upload_indices(render, &self.indices);

        render::draw_triangles(render, self.indices.len() as i32);
    }
}