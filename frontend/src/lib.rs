//! lib.rs – Wasm entry points

mod camera;
mod assets;
mod worker;
mod game;
mod mining;
mod floating_text;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

/* ----------------------------------------------------------------------------
   Small guard so we run the engine only once, even if start_game() is clicked
   multiple times or the Wasm module auto-initialises.
---------------------------------------------------------------------------- */
thread_local! {
    static STARTED: RefCell<bool> = RefCell::new(false);
}

fn boot() -> Result<(), JsValue> {
    STARTED.with(|flag| {
        if *flag.borrow() {
            // already running – ignore duplicate call
            return Ok(());
        }
        *flag.borrow_mut() = true;

        // locate <canvas id="canvas"> and launch the Rust engine
        let canvas: Rc<HtmlCanvasElement> =
            Rc::new(window()
                .ok_or("no window")?
                .document()
                .ok_or("no document")?
                .get_element_by_id("canvas")
                .ok_or("no canvas element")?
                .dyn_into()?);

        game::start_game(canvas)
    })
}

/* --------------------------------------------------------------------------
   ①  Export the function your JS expects
-------------------------------------------------------------------------- */
#[wasm_bindgen]
pub fn start_game() -> Result<(), JsValue> {
    boot()
}

/* --------------------------------------------------------------------------
   ②  Provide an empty auto-start so Wasm binds correctly
       (we DON’T call boot() here – you trigger it from JS by clicking Start)
-------------------------------------------------------------------------- */
#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {
    // do nothing – wait for start_game() from JS
}
