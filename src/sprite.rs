use web_sys::{WebGlProgram, WebGlTexture};

use crate::{gl, object::Object, render::{self, RenderState}};

pub struct Sprite {
    pub x: f32,
    pub y: f32,

    pub vertices: Vec<f32>,
    pub uvs: Vec<f32>,
    pub indices: Vec<u16>,

    pub texture: WebGlTexture,
    pub program: WebGlProgram
}

impl Sprite {
    pub fn new(x: f32, y: f32, texture: WebGlTexture, render: &RenderState, program_option: Option<WebGlProgram>) -> Sprite {

        let program = match program_option {
            Some(program) => program,
            None => render::create_program(&render, None, None).unwrap()
        };

        render::use_program(&render, &program);

        let vertices = gl::BASE_QUAD_VERTS;
        let uvs = gl::BASE_QUAD_UVS;
        let indices = gl::BASE_QUAD_INDICES;

        render::upload_vertices(&render, &vertices);
        render::upload_uvs(&render, &uvs);
        render::upload_indices(&render, &indices);

        Sprite {
            x: x,
            y: y,

            vertices: vertices.to_vec(),
            uvs: uvs.to_vec(),
            indices: indices.to_vec(),

            texture: texture,
            program: program
        }
    }
}

impl Object for Sprite {
    fn update(&mut self, _delta_time: f32) {}

    fn draw(&self, render: &render::RenderState) {
        render::upload_vertices(&render, &self.vertices);
        render::upload_uvs(&render, &self.uvs);
        render::upload_indices(&render, &self.indices);

        /* Attribute vert_texture_coords (vec2) and position (vec3) */
        render::bind_vert_attribs(&render, &self.program);

        /* Uniform texture_sampler (sampler2D) */
        render::bind_frag_uniforms(&render, &self.program, &self.texture);

        /* Finally draw */
        render::draw_triangles(&render, self.indices.len() as i32);
    }
}