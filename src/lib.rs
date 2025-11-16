use crate::app::App;
use crate::debug::log;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::camera::Camera;
use crate::object::Object;
use crate::sprite::Sprite;

mod app;
mod assets;
mod camera;
mod debug;
mod object;
mod render;
mod sprite;

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    let mut app = App::new()?;

    let mut camera = Camera::new(app.canvas.width() as f32, app.canvas.height() as f32);
    camera.rotation = 35.0;

    let camera_pointer = Rc::new(RefCell::new(camera));
    app.cameras.push(camera_pointer.clone());

    let sprite1 = LetsHaveALookCat::new(100.0, 20.0, camera_pointer.clone()).await;
    app.objects.push(Box::new(sprite1));

    let sprite = LetsHaveALookCat::new(400.0, 0.0, camera_pointer.clone()).await;
    app.objects.push(Box::new(sprite));

    for i in 0..200 {
        let banna = Banna::new(i, camera_pointer.clone()).await;
        app.objects.push(Box::new(banna));
    }

    app.start_main_loop();

    Ok(())
}

pub struct LetsHaveALookCat {
    pub sprite: Sprite,

    pub timer: f32,
    pub speed: f32,
}

impl LetsHaveALookCat {
    pub async fn new(x: f32, y: f32, camera: Rc<RefCell<Camera>>) -> LetsHaveALookCat {
        let sprite = Sprite::new(x, y, camera, "assets/catw.png", None).await;

        LetsHaveALookCat {
            sprite,

            speed: 1.0,
            timer: 0.0,
        }
    }

    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }
}

impl Object for LetsHaveALookCat {
    fn update(&mut self, delta_time: f32) {
        self.sprite.update(delta_time);
        self.speed = (self.timer.sin().cos() + 2.) * 3.0;
        self.timer += delta_time * self.speed;
        // self.sprite.x = self.timer.sin() * 70.0;
        self.sprite.rotation = self.timer.sin() * 5.0;
    }

    fn draw(&self, render: &render::Renderer) {
        self.sprite.draw(render);
    }
}

pub struct Banna {
    pub sprite: Sprite,

    pub timer: f32,
    pub i: i32,
}

impl Banna {
    pub async fn new(i: i32, camera: Rc<RefCell<Camera>>) -> Banna {
        let mut sprite = Sprite::new(0.0, 0.0, camera, "assets/banna.png", None).await;
        sprite.scalex = 0.3;
        sprite.scaley = 0.3;

        Banna {
            sprite,
            timer: 0.0,
            i,
        }
    }

    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }
}

impl Object for Banna {
    fn update(&mut self, delta_time: f32) {
        self.sprite.update(delta_time);
        self.timer += delta_time;
        self.sprite.rotation = self.timer.sin() * 25.0;
        // self.sprite.scaley = 0.3 + (self.timer.sin() * 0.3 - (self.i as f32 * 0.3)) * 0.134;

        self.sprite.x = 360.0 * ((self.timer - (self.i as f32 * 0.36)) * 3.0).sin();
        self.sprite.y = 360.0 * ((self.timer - (self.i as f32 * 0.36)) * 3.0).cos();
    }

    fn draw(&self, render: &render::Renderer) {
        self.sprite.draw(render);
    }
}
