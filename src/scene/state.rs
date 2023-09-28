use winit::event::WindowEvent;

pub struct State {
    pub resolution: [f32; 2],
    pub prev_cursor: [f32; 2],
    pub curr_cursor: [f32; 2],
}

impl State {
    pub fn new(resolution: [f32; 2]) -> Self {
        State {
            resolution,
            prev_cursor: [0.0; 2],
            curr_cursor: [0.0; 2],
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.prev_cursor = self.curr_cursor;
                self.curr_cursor = [position.x as f32, position.y as f32];
                false
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.prev_cursor = self.curr_cursor;
    }
}
