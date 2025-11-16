#![allow(unused)]

use crate::log;
use std::{cell::RefCell, rc::Rc};

use js_sys::{Float32Array, Uint16Array};
use wasm_bindgen::JsValue;
use web_sys::{
    HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlVertexArrayObject,
};

use crate::render;

pub const BASE_LEVEL: i32 = 0;

pub const BASE_QUAD_VERTS: [f32; 12] = [
    -1.0, -1.0, 0.0, // bottom-left
    1.0, -1.0, 0.0, // bottom-right
    1.0, 1.0, 0.0, // top-right
    -1.0, 1.0, 0.0, // top-left
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

pub const BASE_VERTEX_SHADER: &str = "#version 300 es

    in vec3 position;
    in vec2 vert_texture_coords;
    out vec2 texture_coords;

    void main() {
        texture_coords = vec2(vert_texture_coords.x, 1.0 - vert_texture_coords.y);
        gl_Position = vec4(position, 1.0);
    }";
pub const BASE_FRAGMENT_SHADER: &str = "#version 300 es
    precision highp float;

    in vec2 texture_coords;
    uniform sampler2D texture_sampler;
    out vec4 output_color;

    void main() {
        output_color = texture(texture_sampler, texture_coords);
    }";

pub struct DrawBuffers {
    pub vertex_buffer: WebGlBuffer,
    pub uv_buffer: WebGlBuffer,
    pub index_buffer: WebGlBuffer,
}

impl DrawBuffers {
    pub fn new(context: &WebGl2RenderingContext) -> DrawBuffers {
        let vertex_buffer =
            DrawBuffers::create_buffer(context, WebGl2RenderingContext::ARRAY_BUFFER).unwrap();
        let uv_buffer =
            DrawBuffers::create_buffer(context, WebGl2RenderingContext::ARRAY_BUFFER).unwrap();
        let index_buffer =
            DrawBuffers::create_buffer(context, WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER)
                .unwrap();

        DrawBuffers {
            vertex_buffer,
            uv_buffer,
            index_buffer,
        }
    }

    pub fn upload_vertices(&self, context: &WebGl2RenderingContext, vertices: &[f32]) {
        DrawBuffers::upload_buffer_f32(
            context,
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.vertex_buffer,
            vertices,
        );
    }

    pub fn upload_uvs(&self, context: &WebGl2RenderingContext, uvs: &[f32]) {
        DrawBuffers::upload_buffer_f32(
            context,
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.uv_buffer,
            uvs,
        );
    }

    pub fn upload_indices(&self, context: &WebGl2RenderingContext, indices: &[u16]) {
        DrawBuffers::upload_buffer_u16(
            context,
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &self.index_buffer,
            indices,
        );
    }

    fn create_buffer(
        context: &WebGl2RenderingContext,
        buffer_type: u32,
    ) -> Result<WebGlBuffer, JsValue> {
        let buffer: WebGlBuffer = context
            .create_buffer()
            .ok_or(JsValue::from_str("Unable to create buffer"))?;
        Ok(buffer)
    }

    fn upload_buffer_f32(
        context: &WebGl2RenderingContext,
        buffer_type: u32,
        buffer: &WebGlBuffer,
        data: &[f32],
    ) {
        context.bind_buffer(buffer_type, Some(buffer));

        unsafe {
            let js_data = Float32Array::view(data);
            context.buffer_data_with_array_buffer_view(
                buffer_type,
                &js_data,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
    }

    fn upload_buffer_u16(
        context: &WebGl2RenderingContext,
        buffer_type: u32,
        buffer: &WebGlBuffer,
        data: &[u16],
    ) {
        context.bind_buffer(buffer_type, Some(buffer));

        unsafe {
            let js_data = Uint16Array::view(data);
            context.buffer_data_with_array_buffer_view(
                buffer_type,
                &js_data,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
    }
}

pub struct Renderer {
    pub context: WebGl2RenderingContext,
    pub quads_buffer: DrawBuffers,

    pub base_program: Option<Rc<WebGlProgram>>,
}

thread_local! {
    pub static RENDERER: RefCell<Option<Rc<Renderer>>> = const { RefCell::new(None) };
}

impl Renderer {
    pub fn new(context: WebGl2RenderingContext) -> Renderer {
        context.enable(WebGl2RenderingContext::BLEND);
        context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        let quads_buffer = DrawBuffers::new(&context);

        let mut renderer = Renderer {
            context,
            quads_buffer,
            base_program: None,
        };

        let base_program = Rc::new(renderer.create_base_program());
        renderer.base_program = Some(base_program);

        renderer
    }

    pub fn create_program(
        &self,
        vertex_source: Option<&str>,
        fragment_source: Option<&str>,
    ) -> WebGlProgram {
        let vertex_source = vertex_source.unwrap_or(BASE_VERTEX_SHADER);
        let fragment_source = fragment_source.unwrap_or(BASE_FRAGMENT_SHADER);

        let vertex_shader = self
            .compile_vertex_shader(vertex_source)
            .unwrap_or(self.compile_base_vertex_shader());
        let fragment_shader = self
            .compile_fragment_shader(fragment_source)
            .unwrap_or(self.compile_base_fragment_shader());

        let program = self.context.create_program().unwrap();
        self.context.attach_shader(&program, &vertex_shader);
        self.context.attach_shader(&program, &fragment_shader);
        self.context.link_program(&program);

        let sucessful = self
            .context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false);
        if (!sucessful) {
            let error_log = self
                .context
                .get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown program linking error".into());
            return self.create_base_program();
        }

        program
    }

    fn create_base_program(&self) -> WebGlProgram {
        let vertex_shader = self.compile_base_vertex_shader();
        let fragment_shader = self.compile_base_fragment_shader();

        let program = self.context.create_program().unwrap();
        self.context.attach_shader(&program, &vertex_shader);
        self.context.attach_shader(&program, &fragment_shader);
        self.context.link_program(&program);

        program
    }

    fn compile_base_vertex_shader(&self) -> WebGlShader {
        self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, BASE_VERTEX_SHADER)
            .unwrap()
    }

    fn compile_vertex_shader(&self, source: &str) -> Option<WebGlShader> {
        self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, source)
    }

    fn compile_base_fragment_shader(&self) -> WebGlShader {
        self.compile_shader(
            WebGl2RenderingContext::FRAGMENT_SHADER,
            BASE_FRAGMENT_SHADER,
        )
        .unwrap()
    }

    fn compile_fragment_shader(&self, source: &str) -> Option<WebGlShader> {
        self.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, source)
    }

    fn compile_shader(&self, shader_type: u32, source: &str) -> Option<WebGlShader> {
        let shader = self.context.create_shader(shader_type).or(None)?;
        self.context.shader_source(&shader, source);
        self.context.compile_shader(&shader);

        let successful = self
            .context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false);
        if !successful {
            let error_log = self
                .context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown shader compiling error".into());
            crate::console_log!("Shader compilation error: {}", error_log);
            crate::console_log!("Shader source: {}", source);
            return None;
        }

        Some(shader)
    }

    pub fn load_texture_image(&self, image: &HtmlImageElement) -> WebGlTexture {
        let texture = self.context.create_texture().unwrap();
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        self.context
            .tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                BASE_LEVEL,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                image,
            );

        self.context
            .generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        // self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        texture
    }

    pub fn set_texture_filtering(&self, texture: &WebGlTexture, antialiasing: bool) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

        let filter = if (antialiasing) {
            WebGl2RenderingContext::LINEAR
        } else {
            WebGl2RenderingContext::NEAREST
        };
        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            filter as i32,
        );

        self.context.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            filter as i32,
        );

        // self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }

    pub fn bind_vert_attribs(&self, buffers: &DrawBuffers, program: &WebGlProgram) {
        // context.bind_vertex_array(Some(&state.vertex_array_object));

        self.context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&buffers.vertex_buffer),
        );
        /* Uniform position (vec4) */
        let position_attrib = self.context.get_attrib_location(program, "position") as u32;
        self.context.enable_vertex_attrib_array(position_attrib);
        self.context.vertex_attrib_pointer_with_i32(
            position_attrib,
            3, /* Uploading 3 floats (x, y, z) */
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        self.context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&buffers.uv_buffer),
        );
        /* Uniform position (vec4) */
        let texture_coords_attrib =
            self.context
                .get_attrib_location(program, "vert_texture_coords") as u32;
        self.context
            .enable_vertex_attrib_array(texture_coords_attrib);
        self.context.vertex_attrib_pointer_with_i32(
            texture_coords_attrib,
            2, /* Uploading 2 floats (x, y) */
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        self.context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&buffers.index_buffer),
        );
    }

    pub fn bind_frag_uniforms(&self, program: &WebGlProgram, texture: &WebGlTexture) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

        let texture_coords_uniform = self
            .context
            .get_uniform_location(program, "texture_sampler")
            .unwrap();
        self.context.uniform1i(Some(&texture_coords_uniform), 0);
    }

    pub fn use_program(&self, program: &WebGlProgram) {
        self.context.use_program(Some(program));
    }

    pub fn use_texture(&self, texture: &WebGlTexture) {
        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
    }

    pub fn draw_triangles(&self, count: i32) {
        self.context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            count,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.context.clear_color(red, green, blue, alpha);
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
}

pub fn with_renderer<T, F>(f: F) -> T
where
    F: FnOnce(&Renderer) -> T,
{
    render::RENDERER.with(|renderer| {
        let renderer_ref = renderer.borrow();
        let renderer_borrow = renderer_ref.as_ref().unwrap();
        f(renderer_borrow)
    })
}
