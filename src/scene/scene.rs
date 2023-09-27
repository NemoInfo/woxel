use winit::event::WindowEvent;

use crate::render::{Camera, CameraController, WgpuContext};

use super::State;

pub struct Scene {
    pub state: State,
    pub camera: Camera,
    pub camera_controller: CameraController,
    // pub voxels: VDB
}

impl Scene {
    pub fn new(context: &WgpuContext) -> Self {
        let aspect = context.config.width as f32 / context.config.height as f32;
        let camera_controller = CameraController::new(0.2);
        Self {
            state: State {},
            camera: Camera::quick_camera(aspect),
            camera_controller,
        }
    }

    pub fn update(&mut self, context: &WgpuContext) {
        self.camera_controller.update_camera(
            &mut self.camera,
            [context.config.width as f32, context.config.height as f32],
        );
    }

    pub fn input(&mut self, event: &WindowEvent) {
        self.camera_controller.process_events(event);
    }
}
