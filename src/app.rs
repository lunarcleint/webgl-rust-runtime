#![allow(unused)]

use crate::{camera, log};

use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Date;
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext, Window};

use crate::camera::Camera;
use crate::object::Object;
use crate::render::{self, Renderer};
use crate::{app, console_log};

pub const BASE_FRAMERATE: f32 = 240.0;

pub struct App {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,

    pub objects: Vec<Box<dyn Object>>,
    pub cameras: Vec<Rc<RefCell<Camera>>>,
    pub renderer: Rc<Renderer>,
    pub framerate: f32,
}

impl App {
    pub fn new() -> Result<App, JsValue> {
        let window = App::query_window()?;
        let document = App::query_document(&window)?;
        let canvas = App::query_canvas(&document)?;

        let context = App::query_gl_context(&canvas)?;
        let renderer = Rc::new(Renderer::new(context));

        render::RENDERER.with(|renderer_mut| {
            *renderer_mut.borrow_mut() = Some(renderer.clone());
        });

        let app = App {
            window,
            document,
            canvas,

            objects: Vec::new(),
            cameras: Vec::new(),
            renderer: renderer.clone(),
            framerate: BASE_FRAMERATE,
        };

        Ok(app)
    }

    pub fn start_main_loop(mut self) {
        let window = self.window.clone();
        let window_pointer = window.clone();

        let func = Rc::new(RefCell::new(None));
        let callback = func.clone();

        let frame_time = (1.0 / &self.framerate) as f64;
        let mut start_time = Date::now();

        *callback.borrow_mut() = Some(Closure::new(move || {
            let current_time = Date::now();
            let delta_time = (current_time - start_time) / 1000.0;

            if (delta_time > frame_time) {
                start_time = current_time;

                self.update(delta_time as f32);
                self.draw(&self.renderer);
            }

            App::schedule_next_frame(&window_pointer, func.borrow().as_ref().unwrap());
        }));

        App::schedule_next_frame(&window, callback.borrow().as_ref().unwrap());
    }

    fn schedule_next_frame(window: &Window, callback: &Closure<dyn FnMut()>) {
        window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .expect("Unable to register request_animation_frame");
    }

    fn query_gl_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, String> {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("antialias"),
            &JsValue::from_bool(true),
        )
        .expect("Unable to set context options");

        let context_query = canvas.get_context_with_context_options("webgl2", &options);
        match context_query {
            Ok(context_option) => match context_option {
                Some(context) => {
                    let js_cast = context.dyn_into::<WebGl2RenderingContext>();
                    match js_cast {
                        Ok(webgl2context) => Ok(webgl2context),
                        Err(_) => Err(String::from("Unable to cast context into JS context")),
                    }
                }
                None => Err(String::from(
                    "Unable to get context (Browser does not support WebGL 2)",
                )),
            },
            Err(_) => Err(String::from(
                "Unable to get context (Browser does not support WebGL 2)",
            )),
        }
    }

    fn query_canvas(document: &Document) -> Result<HtmlCanvasElement, String> {
        let canvas_query = document.get_element_by_id("canvas");
        match canvas_query {
            Some(element) => {
                let js_cast = element.dyn_into::<web_sys::HtmlCanvasElement>();
                match js_cast {
                    Ok(htmlcanvasitem) => Ok(htmlcanvasitem),
                    Err(_) => Err(String::from("Unable to cast canvas into JS canvas")),
                }
            }
            None => Err(String::from("Unable to find canvas")),
        }
    }

    fn query_document(window: &Window) -> Result<Document, String> {
        let document_query = window.document();
        match document_query {
            Some(document) => Ok(document),
            None => Err(String::from("Unable to get web document")),
        }
    }

    fn query_window() -> Result<Window, String> {
        let window_query = web_sys::window();
        match window_query {
            Some(window) => Ok(window),
            None => Err(String::from("Unable to get web window")),
        }
    }
}

impl Object for App {
    fn update(&mut self, delta_time: f32) {
        for object in &mut self.objects {
            object.update(delta_time);
        }
    }

    fn draw(&self, renderer: &render::Renderer) {
        for object in &self.objects {
            object.draw(&self.renderer);
        }

        renderer.clear_color(0.0, 0.0, 0.0, 0.0);

        for camera_ref in &self.cameras {
            let camera = camera_ref.borrow();
            camera.draw(&self.renderer);
        }

        for camera_ref in &self.cameras {
            let mut camera_mut = camera_ref.borrow_mut();
            camera_mut.clear_draws();
        }
    }
}
