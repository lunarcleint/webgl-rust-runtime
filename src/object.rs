use crate::render;

pub trait Object {
    fn update(&mut self, delta_time: f32);
    fn draw(&self, renderer: &render::Renderer);
}