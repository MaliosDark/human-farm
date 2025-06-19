use web_sys::MouseEvent;

/// Simple 2-D camera with pan & zoom.
pub struct Camera {
    pub offset_x: f64,
    pub offset_y: f64,
    pub scale: f64,
    pub dragging: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
}

impl Camera {
    pub fn new() -> Self {
        Self { offset_x: 0.0, offset_y: 0.0, scale: 1.0,
               dragging:false, last_mouse_x:0.0, last_mouse_y:0.0 }
    }

    pub fn start_drag(&mut self, e: &MouseEvent) {
        self.dragging    = true;
        self.last_mouse_x = e.client_x() as f64;
        self.last_mouse_y = e.client_y() as f64;
    }

    pub fn end_drag(&mut self)   { self.dragging = false; }

    pub fn drag(&mut self, e: &MouseEvent) {
        if self.dragging {
            self.offset_x += e.client_x() as f64 - self.last_mouse_x;
            self.offset_y += e.client_y() as f64 - self.last_mouse_y;
            self.last_mouse_x = e.client_x() as f64;
            self.last_mouse_y = e.client_y() as f64;
        }
    }
}
