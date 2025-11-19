#![allow(unused)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlImageElement;
use web_sys::WebGlTexture;

use crate::console_log;
use crate::log;
use crate::render;

#[derive(PartialEq)]
pub struct Image {
    pub html_image: HtmlImageElement,
    pub webl_gl_texture: WebGlTexture,
}

pub struct Assets {
    pub image_cache: HashMap<String, Rc<RefCell<Image>>>,
}

thread_local! {
    pub static ASSETS: RefCell<Assets> = RefCell::new(Assets::new());
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            image_cache: HashMap::new(),
        }
    }

    pub async fn load_image(path: &str) -> Option<Rc<RefCell<Image>>> {
        match Assets::check_cache_image(path).await {
            Some(image_pointer) => Some(image_pointer.clone()),
            None => Assets::cache_image(path).await.ok(),
        }
    }

    async fn check_cache_image(path: &str) -> Option<Rc<RefCell<Image>>> {
        ASSETS.with(|assets| assets.borrow().image_cache.get(path).cloned())
    }

    pub async fn cache_image(path: &str) -> Result<Rc<RefCell<Image>>, JsValue> {
        console_log!("Caching image: {}", path);

        let image = HtmlImageElement::new().unwrap();
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let on_load_closure = wasm_bindgen::closure::Closure::once_into_js(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            });
            image.set_onload(Some(on_load_closure.dyn_ref().unwrap()));

            let on_error_closure = wasm_bindgen::closure::Closure::once_into_js(move || {
                reject.call0(&JsValue::NULL).unwrap();
            });
            image.set_onerror(Some(on_error_closure.dyn_ref().unwrap()));

            image.set_src(path);
        });

        JsFuture::from(promise).await?;

        let image_ref = Assets::generate_texture(image);

        ASSETS.with(|assets| {
            let mut a = assets.borrow_mut();
            a.image_cache.insert(path.to_string(), image_ref.clone());
        });

        Ok(image_ref)
    }

    pub fn clear_image(path: &str) {
        console_log!("Clearing image: {}", path);

        ASSETS.with(|assets| {
            let mut assets_mut = assets.borrow_mut();
            if let Some(image) = assets_mut.image_cache.remove(path) {
                let html_image = &image.borrow().html_image;

                html_image.set_onload(None);
                html_image.set_onerror(None);

                html_image.set_src("");
                html_image.set_attribute("src", "").ok();
            };
        });
    }

    fn generate_texture(image: HtmlImageElement) -> Rc<RefCell<Image>> {
        let webl_gl_texture = render::RENDERER.with(|renderer| {
            let binding = renderer.borrow();
            let renderer_borrow = binding.as_ref().unwrap();
            renderer_borrow.load_texture_image(&image)
        });

        let texture = Image {
            html_image: image,
            webl_gl_texture,
        };
        Rc::new(RefCell::new(texture))
    }
}
