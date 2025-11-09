use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlImageElement;

pub async fn load_image(path: &str) -> Result<HtmlImageElement, JsValue> {
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

    Ok(image)
}