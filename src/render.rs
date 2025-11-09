#![allow(unused)]

use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture, WebGlVertexArrayObject};

use crate::gl;

pub struct RenderState {
    pub context: WebGl2RenderingContext,
    pub vertex_array_object: WebGlVertexArrayObject,

    pub vertex_buffer: WebGlBuffer,
    pub uv_buffer: WebGlBuffer,
    pub index_buffer: WebGlBuffer,
}

pub fn create_renderer(context: WebGl2RenderingContext) -> Result<RenderState, JsValue> {
    let vertex_array_object = gl::create_vertex_array(&context)?;

    let vertex_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ARRAY_BUFFER)?;
    let uv_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ARRAY_BUFFER)?;
    let index_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER)?;

    let render_state = RenderState {
        context,
        vertex_array_object,

        vertex_buffer,
        uv_buffer,
        index_buffer
    };

    Ok(render_state)
}

pub fn create_program(state: &RenderState, vertex_source: Option<&str>, fragment_source : Option<&str>) -> Result<WebGlProgram, JsValue> {
    let context = &state.context;
    let vertex_shader = gl::compile_shader(context, WebGl2RenderingContext::VERTEX_SHADER, vertex_source.unwrap_or(gl::BASE_VERTEX_SHADER))?;
    let fragment_shader = gl::compile_shader(context, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_source.unwrap_or(gl::BASE_FRAGMENT_SHADER))?;

    let program = gl::link_program(context, &vertex_shader, &fragment_shader)?;
    Ok(program)
}

pub fn use_program(state: &RenderState, program: &WebGlProgram) {
    let context = &state.context;
    context.use_program(Some(program));
}

pub fn upload_vertices(state: &RenderState, vertices: &[f32]) {
    gl::upload_buffer_f32(
        &state.context, 
        WebGl2RenderingContext::ARRAY_BUFFER, 
        &state.vertex_buffer,
        vertices
    );
}

pub fn upload_uvs(state: &RenderState, uvs: &[f32]) {
    gl::upload_buffer_f32(
        &state.context, 
        WebGl2RenderingContext::ARRAY_BUFFER, 
        &state.uv_buffer,
        uvs
    );
}

pub fn upload_indices(state: &RenderState, indices: &[u16]) {
    gl::upload_buffer_u16(
        &state.context, 
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, 
        &state.index_buffer,
        indices
    );
}

pub fn bind_vert_attribs(state: &RenderState, program: &WebGlProgram) {
    let context = &state.context;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&state.vertex_buffer));
    /* Uniform position (vec4) */
    let position_attrib = context.get_attrib_location(program, "position") as u32;
    context.enable_vertex_attrib_array(position_attrib);
    context.vertex_attrib_pointer_with_i32(
        position_attrib,
        3, /* Uploading 3 floats (x, y, z) */
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0
    );

    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&state.uv_buffer));
    /* Uniform position (vec4) */
    let texture_coords_attrib = context.get_attrib_location(program, "vert_texture_coords") as u32;
    context.enable_vertex_attrib_array(texture_coords_attrib);
    context.vertex_attrib_pointer_with_i32(
        texture_coords_attrib,
        2, /* Uploading 2 floats (x, y) */
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0
    );
}

pub fn bind_frag_uniforms(state: &RenderState, program: &WebGlProgram, texture: &WebGlTexture) {
    let context = &state.context;
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

    let texture_coords_uniform = context.get_uniform_location(program, "texture_sampler").unwrap();
    context.uniform1i(Some(&texture_coords_uniform), 0);
}

pub fn clear_color(state: &RenderState, red: f32, green: f32, blue: f32, alpha: f32) {
    let context = &state.context;
    context.clear_color(red, green, blue, alpha);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
}

pub fn draw_triangles(state: &RenderState, count: i32) {
    let context = &state.context;

    context.draw_elements_with_i32(
        WebGl2RenderingContext::TRIANGLES,
        count,
        WebGl2RenderingContext::UNSIGNED_SHORT,
        0
    );
}