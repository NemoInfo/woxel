use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::Window,
};

use crate::render::WgpuContext;

pub struct Runtime {
    context: WgpuContext,
    window: Window,
}

impl Runtime {
    pub fn new(context: WgpuContext, window: Window) -> Self {
        Runtime { context, window }
    }

    pub fn main_loop<T>(
        &mut self,
        event: Event<()>,
        _target: &EventLoopWindowTarget<T>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    self.context.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.context.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                match self.context.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.context.resize(self.context.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => {}
        }
    }
}