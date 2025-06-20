// frontend/src/worker.rs

use std::collections::VecDeque;
use js_sys::Math;
use crate::game_loop::{GRID, MOVE_DELAY, WORK_DELAY, MINE_DELAY};

/// Direction the worker moved in last frame
#[derive(Clone, Copy)]
pub enum Direction { Up, Right, Down, Left }

/// Simple grid-based Worker
#[derive(Clone)]
pub struct Worker {
    pub x: usize,
    pub y: usize,
    last_x: usize,
    last_y: usize,
    path: VecDeque<(usize, usize)>,
    ticks_until_move: u8,
    work_ticks: u8,
}

impl Worker {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x, y,
            last_x: x,
            last_y: y,
            path: VecDeque::new(),
            ticks_until_move: 0,
            work_ticks: 0,
        }
    }

    pub fn pause(&mut self, ticks: u8) {
        self.work_ticks = ticks;
    }

    pub fn update(&mut self, has_rock: bool) -> bool {
        self.last_x = self.x;
        self.last_y = self.y;

        if self.work_ticks > 0 {
            self.work_ticks -= 1;
            return false;
        }
        if self.ticks_until_move > 0 {
            self.ticks_until_move -= 1;
            return false;
        }
        self.ticks_until_move = MOVE_DELAY;

        if has_rock {
            self.work_ticks = MINE_DELAY;
            return true;
        }
        if let Some((nx, ny)) = self.path.pop_front() {
            self.x = nx; self.y = ny;
            self.work_ticks = WORK_DELAY;
            return false;
        }

        let mut cands = Vec::new();
        if self.x > 0           { cands.push((self.x - 1, self.y)); }
        if self.x + 1 < GRID   { cands.push((self.x + 1, self.y)); }
        if self.y > 0           { cands.push((self.x, self.y - 1)); }
        if self.y + 1 < GRID   { cands.push((self.x, self.y + 1)); }
        if !cands.is_empty() {
            let idx = (Math::random() * cands.len() as f64).floor() as usize;
            let (nx, ny) = cands[idx];
            self.x = nx; self.y = ny;
            self.work_ticks = WORK_DELAY;
        }
        false
    }

    pub fn direction(&self) -> Direction {
        use Direction::*;
        if self.x > self.last_x      { Right  }
        else if self.x < self.last_x { Left   }
        else if self.y > self.last_y { Down   }
        else if self.y < self.last_y { Up     }
        else                          { Right  }
    }

    pub fn set_target(&mut self, tx: usize, ty: usize) {
        self.path.clear();
        let mut cx = self.x;
        let mut cy = self.y;
        while cy < ty { cy += 1; self.path.push_back((cx, cy)); }
        while cy > ty { cy -= 1; self.path.push_back((cx, cy)); }
        while cx < tx { cx += 1; self.path.push_back((cx, cy)); }
        while cx > tx { cx -= 1; self.path.push_back((cx, cy)); }
    }

    /// Returns (sprite_index, mirror_flag) for rendering.
    /// has_rock → mining frames (3–5), otherwise idle (0–2).
    /// Left direction mirrors the right‐facing sprite.
    pub fn sprite_info(&self, has_rock: bool) -> (usize, bool) {
        use Direction::*;
        match (has_rock, self.direction()) {
            (false, Up)    => (0, false),
            (false, Right) => (1, false),
            (false, Down)  => (2, false),
            (false, Left)  => (1, true),
            (true,  Up)    => (3, false),
            (true,  Right) => (4, false),
            (true,  Down)  => (5, false),
            (true,  Left)  => (4, true),
        }
    }
}
