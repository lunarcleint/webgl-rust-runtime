#![allow(unused)]

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Date};
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, Window};

use crate::camera::Camera;
use crate::object::Object;
use crate::render::{self, Renderer};
use crate::{app, gl};

pub const BASE_FRAMERATE: f32 = 240.0;

pub struct App {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    pub objects: Vec<Box<dyn Object>>,
    pub cameras: Vec<Rc<RefCell<Camera>>>,
    pub render: Rc<Renderer>,
    pub framerate: f32,
}

impl App {
    pub fn new() -> Result<App, JsValue> {
        let window = query_window()?;
        let document = query_document(&window)?;
        let canvas = query_canvas(&document)?;

        let objects = Vec::new();
        let cameras = Vec::new();
        let context = gl::query_gl_context(&canvas)?;
        let state = render::create_renderer(context)?;

        let render = Rc::new(state);

        let framerate = BASE_FRAMERATE;

        let app = App {
            window,
            document,
            canvas,

            objects,
            cameras,
            render,
            framerate
        };

        Ok(app)
    }

    pub fn start_main_loop(mut self) {
        let window = query_window().expect("Unable to get web window");
        let window_clone = window.clone();

        let func = Rc::new(RefCell::new(None));
        let callback = func.clone();

        let frame_time = (1.0 / &self.framerate) as f64;
        let mut start_time = Date::now();

        let render_clone = self.render.clone();

        *callback.borrow_mut() = Some(Closure::new(move || {
            let current_time = Date::now();
            let delta_time = (current_time - start_time) / 1000.0;

            if (delta_time > frame_time) {
                start_time = current_time;
                
                self.update(delta_time as f32);
                self.draw(&render_clone);
            }

            schedule_next_frame(&window_clone, func.borrow().as_ref().unwrap());
        }));

        schedule_next_frame(&window, callback.borrow().as_ref().unwrap());
    }
}

impl Object for App {
    fn update(&mut self, delta_time: f32) {
        
        for object in &mut self.objects {
            object.update(delta_time);
        }
    }

    fn draw(&mut self, render: &render::Renderer) {
        render::clear_color(render, 0.0, 0.0, 0.0, 0.0);

        for object in &mut self.objects {
            object.draw(&self.render);
        }
    }
}

fn schedule_next_frame(window: &Window, callback: &Closure<dyn FnMut()>) {
    window.request_animation_frame(callback.as_ref().unchecked_ref()).expect("Unable to register request_animation_frame");
}

fn query_canvas(document: &Document) -> Result<HtmlCanvasElement, String> {
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

fn query_document(window: &Window) -> Result<Document, String> {
    let document_query = window.document();
    match document_query {
        Some(document) => Ok(document),
        None => Err(String::from("Unable to get web document"))
    }
}

fn query_window() -> Result<Window, String> {
    let window_query = web_sys::window();
    match window_query {
        Some(window) => Ok(window),
        None => Err(String::from("Unable to get web window"))
    }
}