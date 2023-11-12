use egui::{ClippedPrimitive, Color32, FontId, RichText, TexturesDelta};
use egui_wgpu_backend::ScreenDescriptor;
use winit::window::Window;

use crate::scene::Scene;

#[derive(PartialEq)]
pub enum Model {
    Teapot,
    Icosahedron,
    Cube,
}

impl Model {
    fn text(&self) -> String {
        match &self {
            Self::Teapot => "teapot".to_string(),
            Self::Icosahedron => "icosahedron".to_string(),
            Self::Cube => "cube".to_string(),
        }
    }

    pub fn file(&self) -> &'static str {
        match &self {
            Self::Teapot => "utahteapot",
            Self::Icosahedron => "icosahedron",
            Self::Cube => "cube",
        }
    }

    fn rich_text(&self) -> RichText {
        RichText::new(self.text()).font(FontId::proportional(15.0))
    }
}

pub struct EguiDev {
    pub platform: egui_winit_platform::Platform,
    pub model: Model,
}

impl EguiDev {
    pub fn new(platform: egui_winit_platform::Platform) -> Self {
        Self {
            platform,
            model: Model::Teapot,
        }
    }

    pub fn get_frame(
        &mut self,
        scene: &Scene,
        window: &Window,
    ) -> (TexturesDelta, Vec<ClippedPrimitive>, ScreenDescriptor, bool) {
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
                    RichText::new(format!("FPS: {:.0}", scene.state.fps))
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
                });
            });

        let full_output = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);
        let tdelta = full_output.textures_delta;

        (tdelta, paint_jobs, screen_descriptor, model_changed)
    }
}
