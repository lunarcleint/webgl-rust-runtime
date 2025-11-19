#![allow(unused)]

use crate::{log, render::BASE_QUAD_VERTS};
use std::{cell::RefCell, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture};

use crate::{
    app,
    assets::Image,
    console_log,
    object::Object,
    render::{self, BASE_QUAD_INDICES, BASE_QUAD_UVS},
    sprite::Sprite,
};

pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
    pub rotation: f32,

    pub scrollx: f32,
    pub scrolly: f32,

    pub draws: Vec<DrawCall>,
    pub shader: Option<WebGlProgram>,
}

pub struct DrawCall {
    pub texture: WebGlTexture,
    pub program: WebGlProgram,

    pub vertices: Vec<f32>,
    pub count: usize,
}

pub const DEG_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;

impl Camera {
    pub fn new(width: f32, height: f32) -> Camera {
        Camera {
            width,
            height,
            zoom: 1.0,
            rotation: 0.0,
            scrollx: 0.0,
            scrolly: 0.0,
            draws: Vec::new(),
            shader: None,
        }
    }

    pub fn transform_tris(&self, sprite: &Sprite) -> Vec<f32> {
        let mut vertices = render::BASE_QUAD_VERTS;

        for i in (0..vertices.len()).step_by(3) {
            let mut x = vertices[i];
            let mut y = vertices[i + 1];

            /* Sprite transformations */
            x *= sprite.width;
            y *= sprite.height;

            x *= sprite.scalex;
            y *= sprite.scaley;

            if sprite.rotation != 0.0 {
                let radians = -sprite.rotation * DEG_TO_RADIANS;
                let cos_theta = radians.cos();
                let sin_theta = radians.sin();
                let new_x = x * cos_theta - y * sin_theta;
                let new_y = x * sin_theta + y * cos_theta;
                x = new_x;
                y = new_y;
            }

            x += sprite.x;
            y += sprite.y;

            /* Camera transformations */
            x -= self.scrollx;
            y -= self.scrolly;

            x *= self.zoom;
            y *= self.zoom;

            if self.rotation != 0.0 {
                let radians = self.rotation * DEG_TO_RADIANS;
                let cos_theta = radians.cos();
                let sin_theta = radians.sin();
                let new_x = x * cos_theta - y * sin_theta;
                let new_y = x * sin_theta + y * cos_theta;
                x = new_x;
                y = new_y;
            }

            x /= self.width;
            y /= self.height;

            vertices[i] = x;
            vertices[i + 1] = -y;
        }

        vertices.to_vec()
    }

    pub fn clear_draws(&mut self) {
        self.draws.clear();
    }
}

impl Object for Camera {
    fn update(&mut self, delta_time: f32) {}

    fn draw(&self, renderer: &render::Renderer) {
        /* Bind postproccess buffer */
        if let Some(program) = &self.shader {
            renderer.context.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&renderer.post_process.frame_buffer),
            );

            renderer.clear_color(0.0, 0.0, 0.0, 0.0);
        }

        for draw in &self.draws {
            renderer.use_program(&draw.program);
            renderer.use_texture(&draw.texture);

            renderer
                .quads_buffer
                .upload_vertices(&renderer.context, &draw.vertices);

            let mut uvs = Vec::with_capacity(BASE_QUAD_UVS.len() * draw.count);
            for _ in 0..draw.count {
                uvs.extend_from_slice(&BASE_QUAD_UVS);
            }
            renderer.quads_buffer.upload_uvs(&renderer.context, &uvs);

            let mut indices = Vec::with_capacity(BASE_QUAD_INDICES.len() * draw.count);

            for quad in 0..draw.count {
                let base = (quad * 4) as u16;
                for &idx in BASE_QUAD_INDICES.iter() {
                    indices.push(idx + base);
                }
            }
            renderer
                .quads_buffer
                .upload_indices(&renderer.context, &indices);

            renderer.draw_triangles(indices.len() as i32);
        }

        /* Draw postproccess buffer */
        if let Some(program) = &self.shader {
            renderer
                .context
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

            renderer.use_program(program);
            renderer.use_texture(&renderer.post_process.texture);

            renderer
                .quads_buffer
                .upload_vertices(&renderer.context, &BASE_QUAD_VERTS);
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
