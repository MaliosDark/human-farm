use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{HtmlImageElement, Event};

/// Load every URL and call `on_ready(images)` once all are finished.
pub fn load_images(
    urls: &[&str],
    on_ready: impl Fn(Rc<RefCell<Vec<HtmlImageElement>>>) + 'static,
) -> Result<(), JsValue> {
    let images  = Rc::new(RefCell::new(Vec::<HtmlImageElement>::new()));
    let counter = Rc::new(RefCell::new(0usize));
    let total   = urls.len();
    let cb_fn: Rc<dyn Fn(Rc<RefCell<Vec<HtmlImageElement>>>)> = Rc::new(on_ready);

    for url in urls {
        let img  = HtmlImageElement::new()?;
        let imgs = images.clone();
        let cnt  = counter.clone();
        let cb   = cb_fn.clone();

        let onload = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_e| {
            *cnt.borrow_mut() += 1;
            if *cnt.borrow() == total {
                cb(imgs.clone());
            }
        }));
        img.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        img.set_src(url);
        images.borrow_mut().push(img);
    }
    Ok(())
}