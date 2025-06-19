// frontend/src/mining.rs

use std::collections::HashMap;

/// Manages a simple field of rocks, each with a hit-count.
/// Once a rock has been mined N times, it disappears.
#[derive(Clone)]
pub struct RockField {
    /// Map from (x,y) → remaining hits
    pub rocks: HashMap<(usize, usize), u8>,
}

impl RockField {
    /// Build with initial positions; each rock takes 3 hits to destroy.
    pub fn new(positions: &[(usize, usize)]) -> Self {
        let mut rocks = HashMap::new();
        for &pos in positions {
            rocks.insert(pos, 3);
        }
        RockField { rocks }
    }

    /// Called when a worker mines at (x,y). 
    /// Returns `true` if that hit destroyed it.
    pub fn on_mine(&mut self, x: usize, y: usize) -> bool {
        if let Some(count) = self.rocks.get_mut(&(x, y)) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.rocks.remove(&(x, y));
                return true;
            }
        }
        false
    }
}
