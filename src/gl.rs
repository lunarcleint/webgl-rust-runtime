#![allow(unused)]

use js_sys::{Float32Array, Uint16Array};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture, WebGlVertexArrayObject};

pub const BASE_TEXTURE: [u8; 4] = [255, 255, 255, 255];
pub const BASE_TEXTURE_WIDTH: i32 = 1;
pub const BASE_TEXTURE_HEIGHT: i32 = 1;
pub const BASE_TEXTURE_BORDER: i32 = 0;
pub const BASE_LEVEL: i32 = 0;

pub const BASE_QUAD_VERTS: [f32; 12] = [
    -1.0, -1.0, 0.0, // bottom-left
    1.0, -1.0, 0.0, // bottom-right
    1.0,  1.0, 0.0, // top-right
    -1.0,  1.0, 0.0, // top-left
];
pub const BASE_QUAD_UVS: [f32; 8] = [
    0.0, 0.0, // bottom-left
    1.0, 0.0, // bottom-right
    1.0, 1.0, // top-right
    0.0, 1.0, // top-left
];
pub const BASE_QUAD_INDICES: [u16; 6] = [
    0, 1, 2, // first triangle
    2, 3, 0, // second triangle
];

pub const BASE_VERTEX_SHADER: &str = 
    "#version 300 es

    in vec3 position;
    in vec2 vert_texture_coords;
    out vec2 texture_coords;

    void main() {
        texture_coords = vec2(vert_texture_coords.x, 1.0 - vert_texture_coords.y);
        gl_Position = vec4(position, 1.0);
    }"
;
pub const BASE_FRAGMENT_SHADER: &str = 
    "#version 300 es
    precision highp float;

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    out vec4 output_color;

    void main() {
        output_color = texture(texture_sampler, texture_coords);
    }"
;

pub const FUNNY_FRAGMENT_SHADER: &str = 
    "#version 300 es
    precision highp float;

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    out vec4 output_color;

    void main() {
        vec2 uv = texture_coords.xy;

        uv.x += sin((uv.y*4.0)) * 0.02 * 3.0;
        uv.y += cos((uv.x*4.0)) * 0.02 * 3.0;

        output_color = texture(texture_sampler, uv);
    }"
;

pub const BLOOM_FRAGMENT_SHADER: &str = 
    "#version 300 es
    precision highp float;

    #define PI 3.1415926535897932384626433832795
    #define TWO_PI (PI * 2.0)

    #define brightness 2.
    #define threshold .26
    #define directions 360.0
    #define quality 60.0
    #define size 180.0

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    out vec4 output_color;

    void main() {
        vec2 uv = texture_coords.xy;

        uv.x += sin((uv.y*4.0)) * 0.02 * 3.0;
        uv.y += cos((uv.x*4.0)) * 0.02 * 3.0;

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
    }"
;

pub fn load_texture_empty(context: &WebGl2RenderingContext) -> Result<WebGlTexture, JsValue> {
    let texture = context.create_texture().ok_or(JsValue::from_str("Unable to create texture"))?;
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        BASE_LEVEL,
        WebGl2RenderingContext::RGBA as i32,
        BASE_TEXTURE_WIDTH,
        BASE_TEXTURE_HEIGHT,
        BASE_TEXTURE_BORDER,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(&BASE_TEXTURE)
    )?;

    return Ok(texture);
}

pub fn load_texture_image(context: &WebGl2RenderingContext, image: &HtmlImageElement) -> Result<WebGlTexture, JsValue> {
    let texture = context.create_texture().ok_or(JsValue::from_str("Unable to create texture"))?;
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D, 
        WebGl2RenderingContext::TEXTURE_MIN_FILTER, 
        WebGl2RenderingContext::LINEAR as i32
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D, 
        WebGl2RenderingContext::TEXTURE_MAG_FILTER, 
        WebGl2RenderingContext::LINEAR as i32
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D, 
        WebGl2RenderingContext::TEXTURE_WRAP_S, 
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D, 
        WebGl2RenderingContext::TEXTURE_WRAP_T, 
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32
    );

    context.tex_image_2d_with_u32_and_u32_and_html_image_element(
        WebGl2RenderingContext::TEXTURE_2D,
        BASE_LEVEL,
        WebGl2RenderingContext::RGBA as i32,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        &image
    )?;

    context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

    return Ok(texture);
}

pub fn compile_shader(context: &WebGl2RenderingContext, shader_type: u32, source : &str) -> Result<WebGlShader, JsValue> {
    let shader  = context.create_shader(shader_type).ok_or(JsValue::from_str("Unable to create shader"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    let sucessful = context.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false);
    if (!sucessful) {
        let error_log = context.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown shader compiling error".into());
        return Err(JsValue::from_str(&error_log));
    }

    return Ok(shader);
}

pub fn link_program(context: &WebGl2RenderingContext, vertex_shader: &WebGlShader, fragment_shader : &WebGlShader) -> Result<WebGlProgram, JsValue> {
    let program = context.create_program().ok_or(JsValue::from_str("Unable to create program"))?;
    context.attach_shader(&program, vertex_shader);
    context.attach_shader(&program, fragment_shader);
    context.link_program(&program);

    let sucessful = context.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false);
    if (!sucessful){
        let error_log = context.get_program_info_log(&program).unwrap_or_else(|| "Unknown program linking error".into());
        return Err(JsValue::from_str(&error_log));
    }

    return Ok(program);
}

pub fn link_program_multiple(context: &WebGl2RenderingContext, shaders: &[&WebGlShader]) -> Result<WebGlProgram, JsValue> {
    let program = context.create_program().ok_or("Failed to create program")?;
    for shader in shaders {
        context.attach_shader(&program, shader);
    }
    context.link_program(&program);

    let sucessful = context.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS).as_bool().unwrap_or(false);
    if (!sucessful){
        let error_log = context.get_program_info_log(&program).unwrap_or_else(|| "Unknown program linking error".into());
        return Err(JsValue::from_str(&error_log));
    }

    return Ok(program);
}

pub fn create_buffer(context: &WebGl2RenderingContext, buffer_type: u32) -> Result<WebGlBuffer, JsValue> {
    let buffer = context.create_buffer().ok_or(JsValue::from_str("Unable to create buffer"))?;
    context.bind_buffer(buffer_type, Some(&buffer));
    
    return Ok(buffer);
}

pub fn upload_buffer_f32(context: &WebGl2RenderingContext, buffer_type: u32, data: &[f32]) {
    unsafe {
        let js_data = Float32Array::view(&data);

        // Upload the vertex data into the ARRAY_BUFFER on the GPU
        context.buffer_data_with_array_buffer_view(
            buffer_type,
            &js_data,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
}

pub fn upload_buffer_u16(context: &WebGl2RenderingContext, buffer_type: u32, data: &[u16]) {
    unsafe {
        let js_data = Uint16Array::view(&data);

        // Upload the vertex data into the ARRAY_BUFFER on the GPU
        context.buffer_data_with_array_buffer_view(
            buffer_type,
            &js_data,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
}

pub fn create_vertex_array(context: &WebGl2RenderingContext) -> Result<WebGlVertexArrayObject, JsValue> {
    let vertex_array = context.create_vertex_array().ok_or(JsValue::from_str("Unable to create vertex array"))?;
    context.bind_vertex_array(Some(&vertex_array));

    return Ok(vertex_array);
}

pub fn query_gl_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, String> {
    let context_query = canvas.get_context("webgl2");
    return match context_query {
        Ok(context_option) => {
            match context_option {
                Some(context) => {
                    let js_cast = context.dyn_into::<WebGl2RenderingContext>();
                    return match js_cast {
                        Ok(webgl2context) => Ok(webgl2context),
                        Err(_) => Err(String::from("Unable to cast context into JS context"))
                    };
                },
                None => Err(String::from("Unable to get context"))
            }
        }
        Err(_) => Err(String::from("Unable to get context"))
    }
}