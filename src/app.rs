#![allow(unused)]

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Date};
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, Window};

use crate::object::Object;
use crate::render::{self, RenderState};
use crate::{app, gl};

pub struct AppState {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    pub objects: Vec<Box<dyn Object>>,
    pub render: RenderState,
    pub framerate: f32,
}

impl Object for AppState {
    fn update(&mut self, delta_time: f32) {
        for object in &mut self.objects {
            object.update(delta_time);
        }
    }

    fn draw(&self, render: &render::RenderState) {
        render::clear_color(render, 0.0, 0.0, 0.0, 0.0);

        for object in &self.objects {
            object.draw(&self.render);
        }
    }
}

pub fn create_app() -> Result<AppState, JsValue> {
    let window = query_window()?;
    let document = query_document(&window)?;
    let canvas = query_canvas(&document)?;

    let objects = Vec::new();
    let context = gl::query_gl_context(&canvas)?;
    let render = render::create_renderer(context)?;

    const FRAMERATE: f32 = 60.0;

    let app_state = AppState {
        window,
        document,
        canvas,

        objects,
        render,
        framerate: FRAMERATE
    };

    Ok(app_state)
}

pub fn start_loop(mut state: AppState) {
    let window = query_window().expect("Unable to get web window");
    let window_clone = window.clone();

    let func = Rc::new(RefCell::new(None));
    let callback = func.clone();

    let frame_time = (1.0 / &state.framerate) as f64;
    let mut start_time = Date::now();

    *callback.borrow_mut() = Some(Closure::new(move || {
        let current_time = Date::now();
        let delta_time = (current_time - start_time) / 1000.0;

        if (delta_time > frame_time) {
            start_time = current_time;
            
            state.update(delta_time as f32);
            state.draw(&state.render);
        }

        schedule_next_frame(&window_clone, func.borrow().as_ref().unwrap());
    }));

    schedule_next_frame(&window, callback.borrow().as_ref().unwrap());
}

pub fn schedule_next_frame(window: &Window, callback: &Closure<dyn FnMut()>) {
    window.request_animation_frame(callback.as_ref().unchecked_ref()).expect("Unable to register request_animation_frame");
}

pub fn query_canvas(document: &Document) -> Result<HtmlCanvasElement, String> {
    let canvas_query = document.get_element_by_id("canvas");
    match canvas_query {
        Some(element) => {
            let js_cast = element.dyn_into::<web_sys::HtmlCanvasElement>();
            match js_cast {
                Ok(htmlcanvasitem) => Ok(htmlcanvasitem),
                Err(_) => Err(String::from("Unable to cast canvas into JS canvas"))
            }
        },
        None => Err(String::from("Unable to find canvas"))
    }
}

pub fn query_document(window: &Window) -> Result<Document, String> {
    let document_query = window.document();
    match document_query {
        Some(document) => Ok(document),
        None => Err(String::from("Unable to get web document"))
    }
}

pub fn query_window() -> Result<Window, String> {
    let window_query = web_sys::window();
    match window_query {
        Some(window) => Ok(window),
        None => Err(String::from("Unable to get web window"))
    }
}