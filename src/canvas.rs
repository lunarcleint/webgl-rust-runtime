use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, Window};

pub fn query_canvas() -> Result<HtmlCanvasElement, String> {
    let document_query = query_document();
    return match document_query {
        Ok(document) => {
            let canvas_query = document.get_element_by_id("canvas");
            match canvas_query {
                Some(element) => {
                    let js_cast = element.dyn_into::<web_sys::HtmlCanvasElement>();
                    return match js_cast {
                        Ok(htmlcanvasitem) => Ok(htmlcanvasitem),
                        Err(_) => Err(String::from("Unable to cast canvas into JS canvas"))
                    }
                },
                None => Err(String::from("Unable to find canvas"))
            }
        }
        Err(string) => Err(string)
    };
}

pub fn query_document() -> Result<Document, String> {
    let window_query = query_window();
    return match window_query {
        Ok(window) => {
            let document_query = window.document();
            match document_query {
                Some(document) => Ok(document),
                None => Err(String::from("Unable to get web document"))
            }
        },
        Err(string) => Err(string)
    }
}

pub fn query_window() -> Result<Window, String> {
    let window_query = web_sys::window();
    return match window_query {
        Some(window) => Ok(window),
        None => Err(String::from("Unable to get web window"))
    };
}