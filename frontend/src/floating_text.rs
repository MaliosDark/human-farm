/// A little “+X” text that floats upward and then expires.
pub struct FloatText {
    pub x: f64,
    pub y: f64,
    pub text: String,
    lifetime: u8,
}

impl FloatText {
    /// Start at world‐space (x,y) with the given label.
    pub fn new(x: f64, y: f64, text: &str) -> Self {
        FloatText { x, y, text: text.to_string(), lifetime: 60 }
    }

    /// Returns `true` while still alive; also advances its position.
    pub fn update(&mut self) -> bool {
        if self.lifetime == 0 {
            return false;
        }
        self.y -= 0.5;        // float up
        self.lifetime -= 1;
        true
    }
}
