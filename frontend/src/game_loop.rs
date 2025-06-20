// frontend/src/game_loop.rs

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlImageElement, HtmlCanvasElement, CanvasRenderingContext2d as Ctx};
use crate::camera::Camera;
use crate::mining::RockField;
use crate::worker::{Worker, Direction};
use crate::floating_text::FloatText;

pub const GRID: usize      = 10;
pub const TILE_WIDTH: f64  = 64.0;
pub const TILE_HEIGHT: f64 = 32.0;
pub const TILE_SCALE: f64  = 0.5;
pub const HUMAN_SCALE: f64 = 0.10;
pub const ROCK_SCALE: f64  = 0.5;
pub const BLD_SCALE: f64   = 1.0;
pub const WORK_DELAY: u8   = 40;
pub const MINE_DELAY: u8   = 100;
pub const MOVE_DELAY: u8   = 30;

pub fn spawn_render_loop(
    canvas: Rc<HtmlCanvasElement>,
    ctx: Ctx,
    cam: Rc<RefCell<Camera>>,
    workers: Rc<RefCell<Vec<Worker>>>,
    images: Rc<RefCell<Vec<HtmlImageElement>>>,
) -> Result<(), JsValue> {
    // A) Target width from first sprite
    let human_target_w = {
        let imgs = images.borrow();
        imgs[1].natural_width() as f64 * HUMAN_SCALE  // use index 1 (idle_right) as base
    };

    // B) Minimap context
    let minimap_ctx = window().unwrap().document().unwrap()
        .get_element_by_id("minimap-canvas")
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        .and_then(|mc| mc.get_context("2d").ok())
        .and_then(|opt| opt)
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok());

    // C) World state
    let mut rocks     = RockField::new(&[(1,8),(2,7),(3,9)]);
    let buildings     = vec![(2,1),(8,1)];
    let floats        = Rc::new(RefCell::new(Vec::<FloatText>::new()));

    // D) RAF closure
    let raf: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let handle = raf.clone();
    *handle.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let c = cam.borrow();

        // 1) Clear
        ctx.set_fill_style_str("#080808");
        ctx.fill_rect(0.0,0.0, canvas.width() as f64, canvas.height() as f64);

        // 2) Camera
        ctx.save();
        ctx.translate(
            canvas.width() as f64/2.0 + c.offset_x,
            canvas.height() as f64/4.0 + c.offset_y,
        ).unwrap();
        ctx.scale(c.scale, c.scale).unwrap();

        // 3) Floor
        let tw = TILE_WIDTH * TILE_SCALE;
        let th = TILE_HEIGHT * TILE_SCALE;
        for x in 0..GRID {
            for y in 0..GRID {
                let idx = 6 + ((x+y)%3) as usize; // farm_tile at 6,7,8
                let tile = &images.borrow()[idx];
                let (sx,sy) = iso(x as f64,y as f64);
                ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    tile, sx-tw/2.0, sy-th, tw, th
                ).ok();
            }
        }

        // 4) Rocks
        let rock_img = &images.borrow()[9];
        let rw = TILE_WIDTH * ROCK_SCALE;
        let rh = TILE_HEIGHT * ROCK_SCALE;
        for &(rx,ry) in rocks.rocks.keys() {
            let (sx,sy) = iso(rx as f64, ry as f64);
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                rock_img, sx-rw/2.0, sy-rh, rw, rh
            ).ok();
        }

        // 5) Workers
        for worker in workers.borrow_mut().iter_mut() {
            let old = (worker.x,worker.y);
            let on_rock = rocks.rocks.contains_key(&old);

            // mine vs move
            let did_mine = if on_rock {
                worker.pause(MINE_DELAY);
                true
            } else {
                worker.update(on_rock)
            };

            // spawn text
            if did_mine && rocks.on_mine(old.0,old.1) {
                let (fx,fy) = iso(old.0 as f64, old.1 as f64);
                floats.borrow_mut().push(FloatText::new(fx,fy,"+1 Essence"));
            }
            // building block
            if buildings.iter().any(|&b|(b.0==worker.x&&b.1==worker.y)) {
                worker.x = old.0; worker.y = old.1;
                worker.pause(WORK_DELAY);
            }

            // sprite index + mirror
            let (idx, mirror) = worker.sprite_info(on_rock);
            let img_ref = images.borrow();
            let img     = &img_ref[idx];

            // compute dw, dh, dx, dy exactly as you already have:
            let nw    = img.natural_width()  as f64;
            let nh    = img.natural_height() as f64;
            let scale = human_target_w / nw;
            let dw    = nw * scale;
            let dh    = nh * scale;
            let (dx, dy) = iso(worker.x as f64, worker.y as f64);

            // mining glow (unchanged)
            if on_rock {
                ctx.set_fill_style_str("rgba(79,255,79,0.5)");
                ctx.begin_path();
                ctx.arc(dx, dy, 20.0, 0.0, std::f64::consts::PI*2.0).unwrap();
                ctx.fill();
            }

            // draw with mirror flag
            ctx.save();
            if mirror {
                ctx.translate(dx, 0.0).unwrap();
                ctx.scale(-1.0, 1.0).unwrap();
                ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    img, -dw/2.0, dy - dh, dw, dh
                ).ok();
            } else {
                ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    img, dx - dw/2.0, dy - dh, dw, dh
                ).ok();
            }
            ctx.restore();
        }

        // 6) Floating text
        floats.borrow_mut().retain_mut(|ft| {
            if ft.update() {
                ctx.set_fill_style_str("#FFFF00");
                ctx.set_font("16px monospace");
                ctx.fill_text(&ft.text, ft.x, ft.y).ok();
                true
            } else { false }
        });

        // 7) Buildings
        let bld = &images.borrow()[10];
        let bw = TILE_WIDTH * BLD_SCALE;
        let bh = TILE_HEIGHT * 2.0 * BLD_SCALE;
        for &(bx,by) in &buildings {
            let (sx,sy) = iso(bx as f64, by as f64);
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                bld, sx-bw/2.0, sy-bh, bw, bh
            ).ok();
        }

        // 8) Minimap
        if let Some(mm) = &minimap_ctx {
            mm.set_fill_style_str("#080808");
            mm.fill_rect(0.0,0.0,100.0,100.0);
            for x in 0..GRID {
                for y in 0..GRID {
                    let col = if rocks.rocks.contains_key(&(x,y)) {"#4f4"} else {"#222"};
                    mm.set_fill_style_str(col);
                    mm.fill_rect(x as f64*10.0, y as f64*10.0,8.0,8.0);
                }
            }
            for w in workers.borrow().iter() {
                mm.set_fill_style_str("#afa");
                mm.fill_rect(w.x as f64*10.0, w.y as f64*10.0,8.0,8.0);
            }
        }

        // restore & next frame
        ctx.restore();
        window().unwrap()
            .request_animation_frame(
                raf.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            ).unwrap();
    }) as Box<dyn FnMut()>));

    // kick off
    window().unwrap()
        .request_animation_frame(
            handle.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();

    Ok(())
}

fn iso(x: f64, y: f64) -> (f64,f64) {
    (
        (x-y)*(TILE_WIDTH*TILE_SCALE/2.0),
        (x+y)*(TILE_HEIGHT*TILE_SCALE/2.0),
    )
}
