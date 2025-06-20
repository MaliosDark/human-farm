//! lib.rs – Wasm entry points

mod camera;
mod assets;
mod worker;
mod mining;
mod floating_text;
mod game_init;
mod game_loop;

use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use web_sys::window;
use std::cell::RefCell;

thread_local! {
    /// Prevent start_game from running twice
    static STARTED: RefCell<bool> = RefCell::new(false);
    /// If true, auto‐mint every 5s
    static AUTO_MINT: RefCell<bool> = RefCell::new(false);
}

#[wasm_bindgen]
pub fn start_game() -> Result<(), JsValue> {
    STARTED.with(|started| {
        if *started.borrow() {
            return Ok(());
        }
        *started.borrow_mut() = true;

        // Launch the real init
        game_init::start_game()?;

        // If auto‐mint is on, schedule repeated mints
        AUTO_MINT.with(|flag| {
            if *flag.borrow() {
                let cb = Closure::wrap(Box::new(move || {
                    game_init::mint_human();
                }) as Box<dyn Fn()>);
                let _ = window().unwrap()
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        cb.as_ref().unchecked_ref(),
                        5000,
                    );
                cb.forget();
            }
        });

        Ok(())
    })
}

#[wasm_bindgen]
pub fn enable_auto_mint() {
    AUTO_MINT.with(|f| *f.borrow_mut() = true);
}

#[wasm_bindgen]
pub fn disable_auto_mint() {
    AUTO_MINT.with(|f| *f.borrow_mut() = false);
}

// Entry stub so wasm binds correctly.
// We don’t call start_game() here.
#[wasm_bindgen(start)]
pub fn wasm_bindgen_start() {}
