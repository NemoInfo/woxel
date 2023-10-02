use std::io::{stdout, Write};

use log::warn;
use winit::event::WindowEvent;

pub struct State {
    pub resolution: [f32; 2],
    pub prev_cursor: [f32; 2],
    pub curr_cursor: [f32; 2],
    pub cursor_jumped: Option<[f32; 2]>,
    pub time_last_frame: instant::Instant,
    pub initial_time: instant::Instant,
    pub total_time_elapsed_frame: u64,
}

impl State {
    pub fn new(resolution: [f32; 2]) -> Self {
        State {
            resolution,
            prev_cursor: [0.0; 2],
            curr_cursor: [0.0; 2],
            cursor_jumped: None,
            time_last_frame: instant::Instant::now(),
            initial_time: instant::Instant::now(),
            total_time_elapsed_frame: 0,
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
        // @TODO: Add web compatibility
        let total_time_elapsed = self.initial_time.elapsed().as_secs();
        if total_time_elapsed != self.total_time_elapsed_frame {
            let mut stdout = stdout();
            let elapsed = self.time_last_frame.elapsed().as_secs_f64();
            print!("\rFPS: {:.0}", 1. / elapsed);
            stdout
                .flush()
                .unwrap_or_else(|_| warn!("Could not flush stdout"));
        }

        self.prev_cursor = self.curr_cursor;
        self.check_jumped();
        self.time_last_frame = instant::Instant::now();
        self.total_time_elapsed_frame = self.initial_time.elapsed().as_secs();
    }

    pub fn check_jumped(&mut self) {
        if let Some(position) = self.cursor_jumped {
            self.curr_cursor = position;
        }
        self.cursor_jumped = None;
    }
}
