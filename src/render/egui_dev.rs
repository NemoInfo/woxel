use std::time::Instant;

use cgmath::Point3;
use egui::{ClippedPrimitive, Color32, FontId, RichText, TexturesDelta};
use egui_plot::{Bar, BarChart, Plot};
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

#[derive(PartialEq, Clone, Copy)]
pub enum RenderMode {
    Gray,
    Rgb,
    Ray,
}

impl RenderMode {
    fn text(&self) -> String {
        String::from(match &self {
            Self::Gray => "Gray",
            Self::Rgb => "Rgb",
            Self::Ray => "Ray",
        })
    }

    fn rich_text(&self) -> RichText {
        RichText::new(self.text()).font(FontId::proportional(15.0))
    }
}

const FPS_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
pub struct EguiDev {
    pub platform: egui_winit_platform::Platform,
    pub model: Model,
    pub render_mode: RenderMode,
    last_fps_update: Instant,
    time_last_frame: Instant,
    past_fps: Vec<f32>,
    current_fps: f32,
}

impl EguiDev {
    pub fn new(platform: egui_winit_platform::Platform) -> Self {
        Self {
            platform,
            model: Model::Teapot,
            render_mode: RenderMode::Gray,
            last_fps_update: Instant::now(),
            time_last_frame: Instant::now(),
            current_fps: 0.,
            past_fps: vec![0.0; 200],
        }
    }

    pub fn get_frame(
        &mut self,
        scene: &Scene,
        window: &Window,
    ) -> (TexturesDelta, Vec<ClippedPrimitive>, ScreenDescriptor, bool) {
        self.update_fps();

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

                ui.label(
                    RichText::new(format!(
                        "Facing: {}",
                        self.facing(scene.camera.eye, scene.camera.target)
                    ))
                    .font(FontId::proportional(15.0))
                    .color(Color32::WHITE),
                );

                let chart = BarChart::new(
                    self.past_fps
                        .iter()
                        .enumerate()
                        .map(|(index, &time)| Bar::new(index as f64, time as f64 * 1000.))
                        .collect(),
                )
                .color(Color32::LIGHT_GREEN)
                .vertical();

                ui.label(RichText::new("dt histogram (μs)").font(FontId::proportional(15.0)));
                ui.vertical(|ui| {
                    ui.set_height(10.0);
                    Plot::new("dt (ms)")
                        .clamp_grid(true)
                        .y_axis_width(2)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .include_y(20.0)
                        .show(ui, |plot_ui| plot_ui.bar_chart(chart));
                });

                ui.horizontal(|ui| {
                    ui.set_visible(false);
                    ui.label(
                        RichText::new("Camera xyz: -1000.00 -1000.00 -1000.00")
                            .font(FontId::proportional(15.0)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Render: ").font(FontId::proportional(15.0)));
                    ui.selectable_value(
                        &mut self.render_mode,
                        RenderMode::Gray,
                        RenderMode::Gray.rich_text(),
                    )
                    .clicked();
                    ui.selectable_value(
                        &mut self.render_mode,
                        RenderMode::Rgb,
                        RenderMode::Rgb.rich_text(),
                    )
                    .clicked();
                    ui.selectable_value(
                        &mut self.render_mode,
                        RenderMode::Ray,
                        RenderMode::Ray.rich_text(),
                    )
                    .clicked();
                });
            });

        let full_output = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);
        let tdelta = full_output.textures_delta;

        (tdelta, paint_jobs, screen_descriptor, model_changed)
    }

    fn facing(&self, eye: Point3<f32>, target: Point3<f32>) -> String {
        let dir = target - eye;
        let adir = dir.map(|c| c.abs());
        let yzx = (adir.y, adir.z, adir.x).into();
        let zxy = (adir.z, adir.x, adir.y).into();
        let b1 = adir.zip(yzx, |u, v| u >= v);
        let b2 = adir.zip(zxy, |u, v| u >= v);
        let bl = b1.zip(b2, |u, v| u && v);

        let neg = ["-", " "];
        match bl.into() {
            (true, _, _) => format!("{}X", neg[(dir.x > 0.) as usize]),
            (_, true, _) => format!("{}Y", neg[(dir.y > 0.) as usize]),
            (_, _, true) => format!("{}Z", neg[(dir.z > 0.) as usize]),
            _ => unreachable!(),
        }
    }

    fn update_fps(&mut self) {
        let dt = self.time_last_frame.elapsed().as_secs_f32();
        self.past_fps.push(dt);
        self.past_fps.remove(0);
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) > FPS_UPDATE_INTERVAL {
            self.last_fps_update = now;
            let avg_dt = self.past_fps.iter().sum::<f32>() / self.past_fps.len() as f32;
            self.current_fps = 1.0 / avg_dt;
        }

        self.time_last_frame = now;
    }
}
