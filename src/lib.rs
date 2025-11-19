use crate::app::App;
use crate::debug::log;
use crate::render::BASE_VERTEX_SHADER;

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

    let program = render::with_renderer(|renderer| {
        renderer.create_program(None, Some(BLOOM_FRAGMENT_SHADER))
    });

    let mut camera = Camera::new(app.canvas.width() as f32, app.canvas.height() as f32);
    camera.shader = Some(program);
    camera.rotation = 35.0;

    let camera_pointer = Rc::new(RefCell::new(camera));
    app.cameras.push(camera_pointer.clone());

    let sprite1 = LetsHaveALookCat::new(100.0, 20.0, camera_pointer.clone()).await;
    app.objects.push(Box::new(sprite1));

    let sprite = LetsHaveALookCat::new(400.0, 0.0, camera_pointer.clone()).await;
    app.objects.push(Box::new(sprite));

    for i in 0..20 {
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
        let sprite = Sprite::new(x, y, camera, "assets/cat.png", None).await;

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

pub const BLOOM_FRAGMENT_SHADER: &str = "#version 300 es
    precision highp float;

    #define PI 3.1415926535897932384626433832795
    #define TWO_PI (PI * 2.0)

    #define brightness 2.
    #define threshold .7
    #define directions 32.0
    #define quality 6.0
    #define size 20.0

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    out vec4 output_color;

    void main() {
        vec2 uv = texture_coords.xy;

        vec4 color = texture(texture_sampler, uv);
        
        if (brightness <= 0.0 || size <= 0.0) {
            output_color = color;
            return;
        }

        vec4 bloom = vec4(0.0);
        float weightSum = 0.0;

        vec4 highlight = max(color - threshold, 0.0);

        for (float d = 0.0; d < TWO_PI; d += TWO_PI / directions) {
            for (float i = 1.0; i <= float(quality); i++) {
                float offset = (i / float(quality)) * size;
                float x_offset = (sin(d) * offset) / 300.0;
                float y_offset = (cos(d) * offset) / 300.0;
                vec2 sampleUV = clamp(uv + vec2(x_offset, y_offset), vec2(0.0), vec2(1.0));

                vec4 sampleColor = max(texture(texture_sampler, sampleUV) - threshold, 0.0);
                float weight = exp(-2.0 * (i / float(quality)));
                bloom += sampleColor * weight;
                weightSum += weight;
            }
        }

        if (weightSum > 0.0) {
            bloom /= weightSum;
        }

        output_color = color + (bloom * brightness);
    }";
