#![allow(unused)]

use crate::{app, render, sprite::Sprite};

pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
    pub rotation: f32,

    pub scrollx: f32,
    pub scrolly: f32,
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
            vertices[i + 1] = y;
        }

        vertices.to_vec()
    }
}
