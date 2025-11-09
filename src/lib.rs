use wasm_bindgen::prelude::*;

use crate::{render::RenderState, sprite::Sprite};
use crate::object::Object;

mod app;
mod gl;
mod render;
mod assets;
mod object;
mod sprite;
mod debug;

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    let mut app = app::create_app()?;
    let render = &app.render;

    let sprite = LetsHaveALookCat::new(0.0, 0.0, render).await;
    app.objects.push(Box::new(sprite));

    app::start_loop(app);

    Ok(())
}

pub struct LetsHaveALookCat {
    pub sprite: Sprite,

    pub timer: f32,
    pub speed: f32,
}

impl LetsHaveALookCat {
    pub async fn new(x: f32, y: f32, render: &RenderState) -> LetsHaveALookCat {
        /* Load assets */
        let image = assets::load_image("assets/cat.png").await.unwrap();
        let texture = gl::load_texture_image(&render.context, &image).unwrap();
        gl::set_texture_filtering(&render.context, &texture, true);

        LetsHaveALookCat {
            sprite: Sprite::new(x, y, texture, render, None),

            speed: 1.0,
            timer: 0.0,
        }
    }

    pub fn sprite_mut(&mut self) -> &mut Sprite { &mut self.sprite }
    pub fn sprite(&self) -> &Sprite { &self.sprite }
}

impl Object for LetsHaveALookCat {
    fn update(&mut self, delta_time: f32) {
        self.sprite.update(delta_time);
        self.speed = (self.timer.sin().cos() + 2.) * 3.0;
        self.timer += delta_time * self.speed;
    }

    fn draw(&self, render: &render::RenderState) {
        self.sprite.draw(render);
    }
}