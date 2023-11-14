use std::time::Instant;

use egui::{ClippedPrimitive, Color32, FontId, RichText, TexturesDelta};
use egui_wgpu_backend::ScreenDescriptor;
use instant::Duration;
use winit::window::Window;

use crate::scene::Scene;

#[derive(PartialEq)]
pub enum Model {
    Teapot,
    Icosahedron,
    Cube,
    ISS,
}

impl Model {
    fn text(&self) -> String {
        match &self {
            Self::Teapot => "teapot".to_string(),
            Self::Icosahedron => "icosahedron".to_string(),
            Self::Cube => "cube".to_string(),
            Self::ISS => "ISS".to_string(),
        }
    }

    pub fn file(&self) -> &'static str {
        match &self {
            Self::Teapot => "utahteapot",
            Self::Icosahedron => "icosahedron",
            Self::Cube => "cube",
            Self::ISS => "iss",
        }
    }

    fn rich_text(&self) -> RichText {
        RichText::new(self.text()).font(FontId::proportional(15.0))
    }
}

const FPS_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
pub struct EguiDev {
    pub platform: egui_winit_platform::Platform,
    pub model: Model,
    last_fps_update: Instant,
    past_fps: Vec<f32>,
    current_fps: f32,
}

impl EguiDev {
    pub fn new(platform: egui_winit_platform::Platform) -> Self {
        Self {
            platform,
            model: Model::Teapot,
            last_fps_update: Instant::now(),
            current_fps: 0.,
            past_fps: vec![0.0; 30],
        }
    }

    pub fn get_frame(
        &mut self,
        scene: &Scene,
        window: &Window,
    ) -> (TexturesDelta, Vec<ClippedPrimitive>, ScreenDescriptor, bool) {
        self.update_fps(scene.state.fps);

        let screen_descriptor = ScreenDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor() as f32,
        };

        self.platform.begin_frame();
        self.platform
            .context()
            .set_cursor_icon(match scene.state.cursor_grabbed {
                false => egui::CursorIcon::Default,
                true => egui::CursorIcon::None,
            });

        let mut model_changed = false;
        egui::Window::new("Developer tools")
            .title_bar(false)
            .resizable(true)
            .show(&self.platform.context(), |ui| {
                ui.label(
                    RichText::new(format!("FPS: {:.0}", self.current_fps))
                        .color(Color32::from_rgb(7, 173, 51))
                        .font(FontId::proportional(20.0)),
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Model: ").font(FontId::proportional(15.0)));
                    model_changed = ui
                        .selectable_value(&mut self.model, Model::Teapot, Model::Teapot.rich_text())
                        .clicked();
                    model_changed |= ui
                        .selectable_value(
                            &mut self.model,
                            Model::Icosahedron,
                            Model::Icosahedron.rich_text(),
                        )
                        .clicked();
                    model_changed |= ui
                        .selectable_value(&mut self.model, Model::Cube, Model::Cube.rich_text())
                        .clicked();
                    model_changed |= ui
                        .selectable_value(&mut self.model, Model::ISS, Model::ISS.rich_text())
                        .clicked();
                });

                ui.label(
                    RichText::new(format!(
                        "Camera xyz: {:.2} {:.2} {:.2}",
                        scene.camera.eye.x, scene.camera.eye.y, scene.camera.eye.z
                    ))
                    .font(FontId::proportional(15.0)),
                );

                ui.horizontal(|ui| {
                    ui.set_visible(false);
                    ui.label(
                        RichText::new("Camera xyz: -1000.00 -1000.00 -1000.00")
                            .font(FontId::proportional(15.0)),
                    );
                });
            });

        let full_output = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);
        let tdelta = full_output.textures_delta;

        (tdelta, paint_jobs, screen_descriptor, model_changed)
    }

    pub fn update_fps(&mut self, fps: f32) {
        self.past_fps.reverse();
        self.past_fps.push(fps);
        self.past_fps.reverse();
        self.past_fps.pop();
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) > FPS_UPDATE_INTERVAL {
            self.last_fps_update = now;
            self.current_fps = self.past_fps.iter().sum::<f32>() / self.past_fps.len() as f32;
        }
    }
}
