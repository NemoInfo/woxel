use winit::event::WindowEvent;

pub struct State {
    pub resolution: [f32; 2],
    pub prev_cursor: [f32; 2],
    pub curr_cursor: [f32; 2],
    pub cursor_jumped: Option<[f32; 2]>,
    pub time_last_frame: instant::Instant,
}

impl State {
    pub fn new(resolution: [f32; 2]) -> Self {
        State {
            resolution,
            prev_cursor: [0.0; 2],
            curr_cursor: [0.0; 2],
            cursor_jumped: None,
            time_last_frame: instant::Instant::now(),
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.check_jumped();
                self.prev_cursor = self.curr_cursor;
                self.curr_cursor = [position.x as f32, position.y as f32];
                false
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.prev_cursor = self.curr_cursor;
        self.check_jumped();
        self.time_last_frame = instant::Instant::now();
    }

    pub fn check_jumped(&mut self) {
        if let Some(position) = self.cursor_jumped {
            self.curr_cursor = position;
        }
        self.cursor_jumped = None;
    }
}
