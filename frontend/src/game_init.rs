// frontend/src/game_init.rs

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use crate::camera::Camera;
use crate::assets::load_images;
use crate::worker::Worker;
use crate::game_loop::spawn_render_loop;

thread_local! {
    /// Shared workers list for mint_human()
    static WORKERS: RefCell<Option<Rc<RefCell<Vec<Worker>>>>> = RefCell::new(None);
}

/// Called by lib.rs → start_game()
pub fn start_game() -> Result<(), JsValue> {
    // 1) Grab <canvas>
    let doc    = window().unwrap().document().unwrap();
    let raw    = doc.get_element_by_id("canvas").unwrap()
        .dyn_into::<HtmlCanvasElement>()?;
    let canvas = Rc::new(raw);

    // 2) Resize
    let w = window().unwrap().inner_width()?.as_f64().unwrap();
    let h = window().unwrap().inner_height()?.as_f64().unwrap();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    // 3) 2D context
    let ctx = canvas.get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // 4) Camera & input (wheel + pan)
    let cam = Rc::new(RefCell::new(Camera::new()));
    {
        // wheel → zoom
        let cam2 = cam.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::WheelEvent| {
            e.prevent_default();
            let mut c = cam2.borrow_mut();
            c.scale = (c.scale - e.delta_y()*0.001).clamp(0.5,3.0);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }
    {
        // drag → pan
        let cam_d = cam.clone();
        let cvs   = canvas.clone();
        let down = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            cam_d.borrow_mut().start_drag(&e);
            cvs.style().set_property("cursor","grabbing").ok();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", down.as_ref().unchecked_ref())?;
        down.forget();

        let cam_u = cam.clone();
        let cvs_u = canvas.clone();
        let up = Closure::wrap(Box::new(move |_e: web_sys::MouseEvent| {
            cam_u.borrow_mut().end_drag();
            cvs_u.style().set_property("cursor","grab").ok();
        }) as Box<dyn FnMut(_)>);
        window().unwrap()
            .add_event_listener_with_callback("mouseup", up.as_ref().unchecked_ref())?;
        up.forget();

        let cam_m = cam.clone();
        let mv = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            cam_m.borrow_mut().drag(&e);
        }) as Box<dyn FnMut(_)>);
        window().unwrap()
            .add_event_listener_with_callback("mousemove", mv.as_ref().unchecked_ref())?;
        mv.forget();
    }

    // 5) Initialize & store workers
    let workers = Rc::new(RefCell::new(vec![
        Worker::new(6,7),
        Worker::new(6,8),
        Worker::new(6,9),
    ]));
    WORKERS.with(|w| *w.borrow_mut() = Some(workers.clone()));

    // 6) Asset URLs (no left‐facing sprites)
    let urls = [
        "sprites/human_idle_up.png",    // 0
        "sprites/human_idle_right.png", // 1
        "sprites/human_idle_down.png",  // 2
        "sprites/human_work_up.png",    // 3
        "sprites/human_work_right.png", // 4
        "sprites/human_work_down.png",  // 5
        "sprites/farm_tile.png",        // 6
        "sprites/farm_tile_cracked.png",// 7
        "sprites/farm_tile_glow.png",   // 8
        "sprites/rock.png",             // 9
        "sprites/building.png",         // 10
    ];

    // 7) Load assets & start render loop
    load_images(&urls, move |images_rc| {
        spawn_render_loop(
            canvas.clone(),
            ctx.clone(),
            cam.clone(),
            workers.clone(),
            images_rc,
        ).expect("failed to start render loop");
    })?;

    Ok(())
}

/// Exposed for manual or auto‐mint
#[wasm_bindgen]
pub fn mint_human() {
    use js_sys::Math;
    WORKERS.with(|w| {
        if let Some(workers) = &*w.borrow() {
            let x = (Math::random() * crate::game_loop::GRID as f64).floor() as usize;
            workers.borrow_mut().push(Worker::new(x, 0));
        }
    });
}
