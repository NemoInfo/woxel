use log::warn;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::Window,
};

use crate::{render::WgpuContext, scene::Scene};

pub struct Runtime {
    context: WgpuContext,
    window: Window,
    scene: Scene,
}

impl Runtime {
    pub fn new(context: WgpuContext, window: Window, scene: Scene) -> Self {
        Runtime {
            context,
            window,
            scene,
        }
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
            } if window_id == self.window.id() => {
                self.scene.input(&event);
                match event {
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
                        self.scene.state.resolution = (*physical_size).into();
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.context.resize(**new_inner_size);
                        self.scene.state.resolution = self.context.size.into();
                    }
                    WindowEvent::Focused(focus) => {
                        if *focus {
                            let [w, h]: [u32; 2] = self.window.inner_size().into();
                            let pos = [w / 2, h / 2];

                            match self
                                .window
                                .set_cursor_position(Into::<PhysicalPosition<u32>>::into(pos))
                            {
                                Ok(_) => {
                                    self.scene.state.cursor_jumped =
                                        Some([pos[0] as f32, pos[1] as f32]);
                                }
                                Err(_) => warn!("Failed to position cursor"),
                            }

                            self.window
                                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .unwrap_or_else(|_| warn!("Failed to grab cursor"));
                        } else {
                            let [w, h]: [u32; 2] = self.window.inner_size().into();
                            let pos = [w / 2, h / 2];

                            self.window
                                .set_cursor_position(Into::<PhysicalPosition<u32>>::into(pos))
                                .unwrap_or_else(|_| warn!("Failed to position cursor"));

                            self.window
                                .set_cursor_grab(winit::window::CursorGrabMode::None)
                                .unwrap_or_else(|_| warn!("Failed to ungrab cursor"));
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let [x, y]: [f64; 2] = (*position).into();
                        let [w, h]: [u32; 2] = self.window.inner_size().into();

                        if x as u32 == w - 1 || x as u32 == 0 {
                            let pos = [w / 2, y as u32];
                            self.window
                                .set_cursor_position(Into::<PhysicalPosition<u32>>::into(pos))
                                .unwrap_or_else(|_| warn!("Failed to position cursor"));
                            self.scene.state.cursor_jumped = Some([pos[0] as f32, pos[1] as f32]);
                        }
                        if y as u32 == h - 1 || y as u32 == 0 {
                            let pos = [x as u32, h / 2];
                            self.window
                                .set_cursor_position(Into::<PhysicalPosition<u32>>::into(pos))
                                .unwrap_or_else(|_| warn!("Failed to position cursor"));
                            self.scene.state.cursor_jumped = Some([pos[0] as f32, pos[1] as f32]);
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.scene.update();
                match self.context.render(&self.scene) {
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
