#![allow(unused)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlImageElement;

use crate::log;
use crate::console_log;

thread_local! {
    pub static ASSETS: RefCell<Assets> = RefCell::new(Assets::new());
}

pub struct Assets {
    pub image_cache: HashMap<String, Rc<RefCell<HtmlImageElement>>>,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            image_cache: HashMap::new(),
        }
    }

    pub async fn load_image(path: &str) -> Result<Rc<RefCell<HtmlImageElement>>, JsValue> {
        match Assets::check_cache_image(path).await {
            Some(image_pointer) => Ok(image_pointer.clone()),
            None => Ok(Assets::cache_image(path).await?)
        }
    }

    async fn check_cache_image(path: &str) -> Option<Rc<RefCell<HtmlImageElement>>> {
        ASSETS.with(|assets| {
            assets.borrow().image_cache.get(path).cloned()
        })
    }

    pub async fn cache_image(path: &str) -> Result<Rc<RefCell<HtmlImageElement>>, JsValue> {
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
        let pointer = Rc::new(RefCell::new(image));

        ASSETS.with(|assets| {
            let mut assets_mut = assets.borrow_mut();
            assets_mut.image_cache.insert(path.to_string(), pointer.clone());
        });

        Ok(pointer)
    }

    pub fn clear_image(path: &str) {
        console_log!("Clearing image: {}", path);

        ASSETS.with(|assets| {
            let mut assets_mut = assets.borrow_mut();
            assets_mut.image_cache.remove(path);
        });
    }

}