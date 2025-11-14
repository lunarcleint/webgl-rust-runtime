#![allow(unused)]

use crate::{app, gl, sprite::Sprite};

pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
    pub rotation: f32,

    pub scrollx: f32,
    pub scrolly: f32,
}

pub const DEG_TO_RADIANS: f32 = (std::f64::consts::PI / 180.0) as f32;

pub fn transform_tris(sprite: &Sprite, camera: &Camera) -> Vec<f32> {
    let mut vertices = gl::BASE_QUAD_VERTS;

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
        x -= camera.scrollx;
        y -= camera.scrolly;

        x *= camera.zoom;
        y *= camera.zoom;

        if camera.rotation != 0.0 {
            let radians = camera.rotation * DEG_TO_RADIANS;
            let cos_theta = radians.cos();
            let sin_theta = radians.sin();
            let new_x = x * cos_theta - y * sin_theta;
            let new_y = x * sin_theta + y * cos_theta;
            x = new_x;
            y = new_y;
        }

        x /= camera.width;
        y /= camera.height;
        
        vertices[i] = x;
        vertices[i + 1] = y;
    }

    vertices.to_vec()
}