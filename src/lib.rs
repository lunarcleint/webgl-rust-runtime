use wasm_bindgen::prelude::*;
use web_sys::{WebGlTexture, WebGlUniformLocation};

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
    /* Create app */
    let mut app = app::create_app()?;
    let render = &app.render;

    /* Load assets */
    let image = assets::load_image("assets/cat.png").await.unwrap();
    let texture = gl::load_texture_image(&render.context, &image)?;

    /* Create shader */
    let program = render::create_program(&render, None, None)?;
    render::use_program(&render, &program);

    let sprite = LetsHaveALookCat::new(0.0, 0.0, texture, &render);
    app.objects.push(Box::new(sprite));

    app::start_loop(app);

    return Ok(());
}

pub const CAT_VERTEX_SHADER: &str = 
    "#version 300 es

    in vec3 position;
    in vec2 vert_texture_coords;
    out vec2 texture_coords;

    void main() {
        texture_coords = vec2(vert_texture_coords.x, 1.0 - vert_texture_coords.y);
        gl_Position = vec4(position, 1.0);
    }"
;
pub const CAT_FRAGMENT_SHADER: &str = 
    "#version 300 es
    precision highp float;

    #define PI 3.1415926535897932384626433832795
    #define TWO_PI (PI * 2.0)

    uniform float brightness;
    #define threshold .26
    #define directions 360.0
    #define quality 60.0
    #define size 180.0

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    uniform float time;

    out vec4 output_color;

    float rand(vec2 n) {
        return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
    }

    float noise(vec2 n) {
        const vec2 d = vec2(0.0, 1.0);
        vec2 b = floor(n), f = smoothstep(vec2(0.0), vec2(1.0), fract(n));
        return mix(mix(rand(b), rand(b + d.yx), f.x), mix(rand(b + d.xy), rand(b + d.yy), f.x), f.y);
    }

    void main() {
        vec2 uv = texture_coords.xy - 0.5;

        uv *= 1.6;
        uv.xy += vec2(0.45, 0.45);

        uv.x += (sin(time + (uv.y * 4.0)) * 0.02 * 3.0) * (sin(time) + 1.0);
        uv.y += (sin(time * 0.2 + (uv.x * 4.0)) * 0.02 * 3.0) * (cos(time) + 1.0);

        uv.x += (sin(time * 1.5 + (uv.y * 8.0)) * 0.015 * 2.0) * (sin(time * 0.7) + 1.0);
        uv.y += (cos(time * 0.8 + (uv.x * 6.0)) * 0.012 * 2.0) * (cos(time * 1.2) + 1.0);

        uv.x += (cos(time * 0.3 + (uv.y * 2.0)) * 0.025 * 1.5) * (sin(time * 0.5) + 1.0);
        uv.y += (sin(time * 0.4 + (uv.x * 3.0)) * 0.018 * 1.5) * (cos(time * 0.6) + 1.0);

        uv.x += (sin(time * 2.5 + (uv.y * 12.0)) * 0.008 * 1.2);
        uv.y += (cos(time * 2.2 + (uv.x * 10.0)) * 0.006 * 1.2);

        uv.x += (sin(time * 1.2 + (uv.y * 5.0 + 1.57)) * 0.01 * 1.8) * (cos(time * 0.9) + 1.0);
        uv.y += (cos(time * 1.1 + (uv.x * 7.0 - 0.78)) * 0.009 * 1.8) * (sin(time * 1.3) + 1.0);

        vec2 wateruv = texture_coords.xy;
        wateruv.y += time * 0.1;
        vec2 dst_offset = (vec4(noise(wateruv * vec2(30))).xy - vec2(0.3, 0.3)) * 1.0 * 0.03;

        vec2 circ1 = vec2(sin(time), cos(time * 1.2)) * 0.35;
        vec2 circ2 = vec2(cos(time * 0.8), sin(time * 1.5)) * 0.28;
        vec2 circ3 = vec2(sin(time * 1.7), cos(time * 0.6)) * 0.42;
        
        vec2 flow = vec2(0.0);
        flow += normalize(circ1 - uv) * 0.015 * sin(time * 3.0 + uv.x * 10.0);
        flow += normalize(circ2 - uv) * 0.012 * cos(time * 2.5 + uv.y * 8.0);
        flow += normalize(circ3 - uv) * 0.018 * sin(time * 4.0 + (uv.x + uv.y) * 6.0);
        
        vec2 turbulence = vec2(
            noise(uv * 4.0 + time) - 0.5,
            noise(uv * 4.0 + time + 100.0) - 0.5
        ) * 0.02;

        vec2 circ = vec2(cos(time), sin(time))*.1 + flow + turbulence * 7.;

        vec2 final_uv = uv + dst_offset;
        if (final_uv.x <= 0.01 + circ.x) discard;
        if (final_uv.x >= 0.99 + circ.x) discard;

        if (final_uv.y <= 0.01 + circ.y) discard;
        if (final_uv.y >= 0.99 + circ.y) discard;

        vec4 color = texture(texture_sampler, final_uv - circ);
        
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
                vec2 sampleUV = clamp(uv - circ + vec2(x_offset, y_offset), vec2(0.0), vec2(1.0));

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
    }"
;

pub struct LetsHaveALookCat {
    pub sprite: Sprite,

    pub timer: f32,
    pub speed: f32,

    // pub time_uniform: WebGlUniformLocation,
    // pub bloom_uniform: WebGlUniformLocation
}

impl LetsHaveALookCat {
    pub fn new(x: f32, y: f32, texture: WebGlTexture, render: &RenderState) -> LetsHaveALookCat {
        let program = render::create_program(&render, None, None).unwrap();
        // let time_uniform = &render.context.get_uniform_location(&program, "time").unwrap();
        // let bloom_uniform = &render.context.get_uniform_location(&program, "brightness").unwrap();

        LetsHaveALookCat {
            sprite: Sprite::new(x, y, texture, &render, Some(program)),
            // time_uniform: time_uniform.to_owned(),
            // bloom_uniform: bloom_uniform.to_owned(),

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
        // render.context.uniform1f(Some(&self.time_uniform), self.timer);
        // render.context.uniform1f(Some(&self.bloom_uniform), (self.timer/10.0).sin()*2.0);
        self.sprite.draw(render);
    }
}