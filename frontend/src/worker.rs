use std::collections::VecDeque;
use js_sys::Math;
use crate::game::{GRID, MOVE_DELAY, WORK_DELAY, MINE_DELAY};

/// Direction the worker moved in last frame
#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

/// A simple grid‐based Worker that walks randomly across the map,
/// mines rocks, pauses, and now remembers its last position to
/// expose a `.direction()` method.
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
    /// Create a new Worker at grid position (x, y).
    /// Initializes last_x/last_y to the same position.
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            last_x: x,
            last_y: y,
            path: VecDeque::new(),
            ticks_until_move: 0,
            work_ticks: 0,
        }
    }

    /// Public API to force a pause (e.g. bumped into a building).
    pub fn pause(&mut self, ticks: u8) {
        self.work_ticks = ticks;
    }

    /// Called each frame.
    /// - `has_rock` indicates rock presence.
    /// Returns `true` if this frame completed a mining action.
    pub fn update(&mut self, has_rock: bool) -> bool {
        // 1) Remember previous position for direction detection:
        self.last_x = self.x;
        self.last_y = self.y;

        // 2) Working / mining cooldown
        if self.work_ticks > 0 {
            self.work_ticks -= 1;
            return false;
        }

        // 3) Movement delay
        if self.ticks_until_move > 0 {
            self.ticks_until_move -= 1;
            return false;
        }

        // 4) Reset move timer
        self.ticks_until_move = MOVE_DELAY;

        // 5) If standing on a rock, “mine” it
        if has_rock {
            self.work_ticks = MINE_DELAY;
            return true;
        }

        // 6) If following a path, step along it
        if let Some((nx, ny)) = self.path.pop_front() {
            self.x = nx;
            self.y = ny;
            self.work_ticks = WORK_DELAY;
            return false;
        }

        // 7) Wander randomly one tile
        let mut candidates = Vec::new();
        if self.x > 0                { candidates.push((self.x - 1, self.y)); }
        if self.x + 1 < GRID        { candidates.push((self.x + 1, self.y)); }
        if self.y > 0                { candidates.push((self.x, self.y - 1)); }
        if self.y + 1 < GRID        { candidates.push((self.x, self.y + 1)); }

        if !candidates.is_empty() {
            let idx = (Math::random() * (candidates.len() as f64)).floor() as usize;
            let (nx, ny) = candidates[idx];
            self.x = nx;
            self.y = ny;
            self.work_ticks = WORK_DELAY;
        }

        false
    }

    /// Returns the direction of movement this frame.
    pub fn direction(&self) -> Direction {
        use Direction::*;
        if self.x > self.last_x      { Right }
        else if self.x < self.last_x { Left  }
        else if self.y > self.last_y { Down  }
        else if self.y < self.last_y { Up    }
        else                          { Right } // default idle facing right
    }

    /// Optional: set a manual path for the worker
    pub fn set_target(&mut self, tx: usize, ty: usize) {
        self.path.clear();
        let mut cx = self.x;
        let mut cy = self.y;
        while cy < ty {
            cy += 1;
            self.path.push_back((cx, cy));
        }
        while cy > ty {
            cy -= 1;
            self.path.push_back((cx, cy));
        }
        while cx < tx {
            cx += 1;
            self.path.push_back((cx, cy));
        }
        while cx > tx {
            cx -= 1;
            self.path.push_back((cx, cy));
        }
    }
}
