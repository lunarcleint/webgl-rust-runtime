use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

mod canvas;
mod gl;
mod assets;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    let canvas_query = canvas::query_canvas();

    match canvas_query {
        Ok(canvas) => {
            let context_query = gl::query_gl_context(&canvas);
            match context_query {
                Ok(context) => {
                    let image = assets::load_image("assets/cat.png").await.unwrap();
                    let texture = gl::load_texture_image(&context, &image)?;

                    /* Rest will go into a render.rs */
                    let vertex_source = gl::BASE_VERTEX_SHADER;
                    let fragment_source = gl::BLOOM_FRAGMENT_SHADER;

                    let vertex_shader = gl::compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, vertex_source)?;
                    let fragment_shader = gl::compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_source)?;

                    let program = gl::link_program(&context, &vertex_shader, &fragment_shader)?;

                    context.use_program(Some(&program));
                    context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

                    let verts = gl::BASE_QUAD_VERTS;
                    let uvs = gl::BASE_QUAD_UVS;
                    let indices = gl::BASE_QUAD_INDICES;

                    let _vao = gl::create_vertex_array(&context)?;

                    let _vertex_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ARRAY_BUFFER)?;
                    gl::upload_buffer_f32(&context, WebGl2RenderingContext::ARRAY_BUFFER, &verts);

                    /* Uniform position (vec4) */
                    let position_attrib = context.get_attrib_location(&program, "position") as u32;
                    context.enable_vertex_attrib_array(position_attrib);
                    context.vertex_attrib_pointer_with_i32(
                        position_attrib,
                        3, /* Uploading 3 floats (x, y, z) */
                        WebGl2RenderingContext::FLOAT,
                        false,
                        0,
                        0
                    );

                    let _uv_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ARRAY_BUFFER)?;
                    gl::upload_buffer_f32(&context, WebGl2RenderingContext::ARRAY_BUFFER, &uvs);

                    /* Uniform position (vec4) */
                    let texture_coords_attrib = context.get_attrib_location(&program, "vert_texture_coords") as u32;
                    context.enable_vertex_attrib_array(texture_coords_attrib);
                    context.vertex_attrib_pointer_with_i32(
                        texture_coords_attrib,
                        2, /* Uploading 2 floats (x, y) */
                        WebGl2RenderingContext::FLOAT,
                        false,
                        0,
                        0
                    );

                    let _index_buffer = gl::create_buffer(&context, WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER)?;
                    gl::upload_buffer_u16(&context, WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, &indices);

                    /* Uniform texture_sampler (sampler2D) */
                    context.active_texture(WebGl2RenderingContext::TEXTURE0);
                    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

                    let texture_coords_uniform = context.get_uniform_location(&program, "texture_sampler").unwrap();
                    context.uniform1i(Some(&texture_coords_uniform), 0);

                    /* Finally draw */
                    context.clear_color(1.0, 0.0, 0.0, 1.0);
                    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                    context.draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        indices.len() as i32,
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        0
                    );
                },
                Err(string) => alert(&string),
            }
        }
        Err(string) => return Err(JsValue::from_str(&string))
    }

    return Ok(());
}

fn _check_gl_errors(context: &WebGl2RenderingContext, location: &str) {
    let error = context.get_error();
    match error {
        WebGl2RenderingContext::NO_ERROR => (),
        WebGl2RenderingContext::INVALID_ENUM => alert(&format!("GL INVALID_ENUM at {}", location)),
        WebGl2RenderingContext::INVALID_VALUE => alert(&format!("GL INVALID_VALUE at {}", location)),
        WebGl2RenderingContext::INVALID_OPERATION => alert(&format!("GL INVALID_OPERATION at {}", location)),
        WebGl2RenderingContext::INVALID_FRAMEBUFFER_OPERATION => alert(&format!("GL INVALID_FRAMEBUFFER_OPERATION at {}", location)),
        WebGl2RenderingContext::OUT_OF_MEMORY => alert(&format!("GL OUT_OF_MEMORY at {}", location)),
        _ => alert(&format!("GL Unknown error: {} at {}", error, location)),
    }
}