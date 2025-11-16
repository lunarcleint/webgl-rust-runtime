use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

fn _check_gl_errors(context: &WebGl2RenderingContext, location: &str) {
    let error = context.get_error();
    match error {
        WebGl2RenderingContext::NO_ERROR => (),
        WebGl2RenderingContext::INVALID_ENUM => alert(&format!("GL INVALID_ENUM at {}", location)),
        WebGl2RenderingContext::INVALID_VALUE => {
            alert(&format!("GL INVALID_VALUE at {}", location))
        }
        WebGl2RenderingContext::INVALID_OPERATION => {
            alert(&format!("GL INVALID_OPERATION at {}", location))
        }
        WebGl2RenderingContext::INVALID_FRAMEBUFFER_OPERATION => {
            alert(&format!("GL INVALID_FRAMEBUFFER_OPERATION at {}", location))
        }
        WebGl2RenderingContext::OUT_OF_MEMORY => {
            alert(&format!("GL OUT_OF_MEMORY at {}", location))
        }
        _ => alert(&format!("GL Unknown error: {} at {}", error, location)),
    }
}

/* https://rustwasm.github.io/docs/wasm-bindgen/examples/console-log.html */
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ({
        log(&format!($($t)*));
    })
}
