// frontend/src/game.rs

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement,
    MouseEvent, WheelEvent,
};

use crate::camera::Camera;
use crate::assets::load_images;
use crate::worker::{Worker, Direction};
use crate::mining::RockField;
use crate::floating_text::FloatText;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

pub const GRID: usize      = 10;
pub const TILE_WIDTH: f64  = 64.0;
pub const TILE_HEIGHT: f64 = 32.0;
pub const TILE_SCALE: f64  = 0.5;
pub const HUMAN_SCALE: f64 = 0.10;
pub const ROCK_SCALE: f64  = 0.5;
pub const BLD_SCALE: f64   = 1.0;

// Pause durations (in frames)
pub const WORK_DELAY: u8   = 40;
pub const MINE_DELAY: u8   = 100;
pub const MOVE_DELAY: u8   = 30;

pub fn start_game(canvas: Rc<HtmlCanvasElement>) -> Result<(), JsValue> {
    // ─── Canvas setup ───────────────────────────────────────────────
    let w = window().unwrap().inner_width()?.as_f64().unwrap();
    let h = window().unwrap().inner_height()?.as_f64().unwrap();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?
        .unwrap().dyn_into()?;

    // ─── Minimap setup (optional) ──────────────────────────────────
    let minimap_ctx: Option<CanvasRenderingContext2d> = {
        let doc = window().unwrap().document().unwrap();
        doc.get_element_by_id("minimap-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            .and_then(|mc| mc.get_context("2d").ok())
            .and_then(|opt| opt)
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    };

    // ─── Camera & Input ─────────────────────────────────────────────
    let cam = Rc::new(RefCell::new(Camera::new()));

    // wheel → zoom
    {
        let cam_wheel = cam.clone();
        let cb = Closure::wrap(Box::new(move |e: WheelEvent| {
            e.prevent_default();
            let mut c = cam_wheel.borrow_mut();
            c.scale = (c.scale - e.delta_y() * 0.001).clamp(0.5, 3.0);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback(
            "wheel", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }

    // drag → pan
    {
        let cam_d = cam.clone();
        let cvs = canvas.clone();
        // mousedown
        let down = Closure::wrap(Box::new(move |e: MouseEvent| {
            cam_d.borrow_mut().start_drag(&e);
            cvs.as_ref().style().set_property("cursor", "grabbing").ok();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback(
            "mousedown", down.as_ref().unchecked_ref())?;
        down.forget();

        // mouseup
        let cam_u = cam.clone();
        let cvs_u = canvas.clone();
        let up = Closure::wrap(Box::new(move |_e: MouseEvent| {
            cam_u.borrow_mut().end_drag();
            cvs_u.as_ref().style().set_property("cursor", "grab").ok();
        }) as Box<dyn FnMut(_)>);
        window().unwrap().add_event_listener_with_callback(
            "mouseup", up.as_ref().unchecked_ref())?;
        up.forget();

        // mousemove
        let cam_m = cam.clone();
        let mv = Closure::wrap(Box::new(move |e: MouseEvent| {
            cam_m.borrow_mut().drag(&e);
        }) as Box<dyn FnMut(_)>);
        window().unwrap().add_event_listener_with_callback(
            "mousemove", mv.as_ref().unchecked_ref())?;
        mv.forget();
    }

    // ─── Entities ───────────────────────────────────────────────────
    let workers   = Rc::new(RefCell::new(vec![
        Worker::new(6, 7),
        Worker::new(6, 8),
        Worker::new(6, 9),
    ]));
    let rocks     = RockField::new(&[(1, 8), (2, 7), (3, 9)]);
    let buildings = Rc::new(vec![(2, 1), (8, 1)]);
    let floats    = Rc::new(RefCell::new(Vec::<FloatText>::new()));

    // ─── Load assets & render loop ──────────────────────────────────
    let urls = [
        "sprites/human_idle_up.png",    // 0
        "sprites/human_idle_right.png", // 1
        "sprites/human_idle_down.png",  // 2
        "sprites/human_work_up.png",    // 3
        "sprites/human_work_right.png", // 4
        "sprites/farm_tile.png",        // 5
        "sprites/farm_tile_cracked.png",// 6
        "sprites/farm_tile_glow.png",   // 7
        "sprites/rock.png",             // 8
        "sprites/building.png",         // 9
    ];

    load_images(&urls, move |imgs: Rc<RefCell<Vec<HtmlImageElement>>>| {
        let canvas      = canvas.clone();
        let cam         = cam.clone();
        let ctx         = ctx.clone();
        let wkr         = workers.clone();
        let mut rocks   = rocks.clone();
        let blds        = buildings.clone();
        let floats      = floats.clone();
        let minimap_ctx = minimap_ctx.clone();

        // closure for requestAnimationFrame
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
            Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let c = cam.borrow();

            // 1) clear main canvas
            ctx.set_fill_style_str("#080808");
            ctx.fill_rect(0.0, 0.0,
                canvas.width() as f64,
                canvas.height() as f64
            );

            // 2) camera transform
            ctx.save();
            ctx.translate(
                canvas.width() as f64 / 2.0 + c.offset_x,
                canvas.height() as f64 / 4.0 + c.offset_y,
            ).unwrap();
            ctx.scale(c.scale, c.scale).unwrap();

            // 3) floor
            let tw = TILE_WIDTH * TILE_SCALE;
            let th = TILE_HEIGHT * TILE_SCALE;
            for x in 0..GRID {
                for y in 0..GRID {
                    let variant = ((x + y) % 3) as usize;
                    if let Some(tile) = imgs.borrow().get(5 + variant) {
                        let (sx, sy) = iso(x as f64, y as f64);
                        ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            tile,
                            sx - tw / 2.0, sy - th,
                            tw, th
                        ).ok();
                    }
                }
            }

            // 4) rocks
            if let Some(rock_img) = imgs.borrow().get(8) {
                let rw = TILE_WIDTH * ROCK_SCALE;
                let rh = TILE_HEIGHT * ROCK_SCALE;
                for &pos in rocks.rocks.keys() {
                    let (sx, sy) = iso(pos.0 as f64, pos.1 as f64);
                    ctx.draw_image_with_html_image_element_and_dw_and_dh(
                        rock_img,
                        sx - rw / 2.0, sy - rh,
                        rw, rh
                    ).ok();
                }
            }

            // 5) workers & mining
            for worker in wkr.borrow_mut().iter_mut() {
                let old     = (worker.x, worker.y);
                let on_rock = rocks.rocks.contains_key(&old);

                // mining vs moving
                let mined = if on_rock {
                    worker.pause(MINE_DELAY);
                    true
                } else {
                    worker.update(on_rock)
                };

                // spawn floating text on mine
                if mined && rocks.on_mine(old.0, old.1) {
                    let (fx, fy) = iso(old.0 as f64, old.1 as f64);
                    floats.borrow_mut().push(
                        FloatText::new(fx, fy, "+1 Essence")
                    );
                }

                // block through buildings
                if blds.iter().any(|&b| b == (worker.x, worker.y)) {
                    worker.x = old.0;
                    worker.y = old.1;
                    worker.pause(WORK_DELAY);
                }

                // 5a) pick which sprite index and whether to mirror
                let dir = worker.direction();
                let (idx, mirror) = match (on_rock, dir) {
                    (false, Direction::Up)    => (0, false),
                    (false, Direction::Right) => (1, false),
                    (false, Direction::Down)  => (2, false),
                    (false, Direction::Left)  => (1, true),
                    (true,  Direction::Up)    => (3, false),
                    (true,  Direction::Right) => (4, false),
                    (true,  Direction::Down)  => (3, true),  // reuse up
                    (true,  Direction::Left)  => (4, true),
                };

                // 5b) draw the selected human image
                if let Some(img) = imgs.borrow().get(idx) {
                    let dw = img.width()  as f64 * HUMAN_SCALE;
                    let dh = img.height() as f64 * HUMAN_SCALE;
                    let (dx, dy) = iso(worker.x as f64, worker.y as f64);

                    // mining glow
                    if on_rock {
                        ctx.set_fill_style_str("rgba(79,255,79,0.5)");
                        ctx.begin_path();
                        ctx.arc(dx, dy, 20.0, 0.0,
                            2.0 * std::f64::consts::PI
                        ).unwrap();
                        ctx.fill();
                    }

                    // mirror if needed
                    ctx.save();
                    if mirror {
                        ctx.translate(dx, 0.0).unwrap();
                        ctx.scale(-1.0, 1.0).unwrap();
                        ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            img,
                            -dw / 2.0,
                            dy - dh,
                            dw,
                            dh
                        ).ok();
                    } else {
                        ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            img,
                            dx - dw / 2.0,
                            dy - dh,
                            dw,
                            dh
                        ).ok();
                    }
                    ctx.restore();
                }
            }

            // 6) floating text
            floats.borrow_mut().retain_mut(|ft| {
                if ft.update() {
                    ctx.set_fill_style_str("#FFFF00");
                    ctx.set_font("16px monospace");
                    ctx.fill_text(&ft.text, ft.x, ft.y).ok();
                    true
                } else {
                    false
                }
            });

            // 7) buildings on top
            if let Some(bld) = imgs.borrow().get(9) {
                let bw = TILE_WIDTH * BLD_SCALE;
                let bh = TILE_HEIGHT * 2.0 * BLD_SCALE;
                for &(bx, by) in blds.iter() {
                    let (sx, sy) = iso(bx as f64, by as f64);
                    ctx.draw_image_with_html_image_element_and_dw_and_dh(
                        bld,
                        sx - bw / 2.0, sy - bh,
                        bw, bh
                    ).ok();
                }
            }

            // 8) minimap
            if let Some(mm) = &minimap_ctx {
                mm.set_fill_style_str("#080808");
                mm.fill_rect(0.0, 0.0, 100.0, 100.0);
                for x in 0..GRID {
                    for y in 0..GRID {
                        let mx = x as f64 * 10.0;
                        let my = y as f64 * 10.0;
                        let col = if rocks.rocks.contains_key(&(x, y)) {
                            "#4f4"
                        } else {
                            "#222"
                        };
                        mm.set_fill_style_str(col);
                        mm.fill_rect(mx, my, 8.0, 8.0);
                    }
                }
                for w in wkr.borrow().iter() {
                    let mx = w.x as f64 * 10.0;
                    let my = w.y as f64 * 10.0;
                    mm.set_fill_style_str("#afa");
                    mm.fill_rect(mx, my, 8.0, 8.0);
                }
            }

            // restore & loop
            ctx.restore();
            window().unwrap()
                .request_animation_frame(
                    f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                ).unwrap();
        }) as Box<dyn FnMut()>));

        // start loop
        window().unwrap()
            .request_animation_frame(
                g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            ).unwrap();
    })?;

    Ok(())
}

/// Convert grid coords to isometric screen coords
fn iso(x: f64, y: f64) -> (f64, f64) {
    (
        (x - y) * (TILE_WIDTH * TILE_SCALE / 2.0),
        (x + y) * (TILE_HEIGHT * TILE_SCALE / 2.0),
    )
}
