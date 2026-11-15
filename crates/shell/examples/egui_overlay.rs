use egui_phosphor::regular as ic;
use ume_render::Color;
use ume_shell::{run, Config, Frame, Game, Key};

struct Demo {
    ready: bool,
    frame: u32,
    t: f32,
    volume: f32,
    glass: bool,
}

impl Game for Demo {
    fn frame(&mut self, f: &mut Frame) {
        if f.input.is_pressed(Key::Escape) {
            f.quit = true;
        }
        self.t = f.time as f32;
        self.frame += 1;
        if let (Ok(path), true) = (std::env::var("UME_SHOT"), self.frame == 60) {
            f.screenshot = Some(path);
        }
        f.clear = [0.04, 0.08, 0.16, 1.0];
        let [w, h] = f.size;
        f.canvas
            .rect(0.0, h * 0.75, w, h * 0.25, Color::rgba(40, 60, 90, 255));
        let x = w * 0.5 + (self.t * 1.2).sin() * w * 0.25;
        f.canvas
            .circle([x, h * 0.55], h * 0.1, Color::rgba(230, 90, 40, 255));
        f.canvas
            .line([0.0, h * 0.75], [w, h * 0.75], 4.0, Color::WHITE);
    }

    fn ui(&mut self, ctx: &egui::Context) {
        if !self.ready {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            ctx.set_fonts(fonts);
            self.ready = true;
        }
        egui::Window::new(format!("{} Settings", ic::SLIDERS_HORIZONTAL))
            .default_pos([40.0, 40.0])
            .show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.volume, 0.0..=1.0)
                        .text(format!("{} Volume", ic::SPEAKER_HIGH)),
                );
                ui.checkbox(&mut self.glass, format!("{} Glass HUD", ic::DROP));
                ui.separator();
                ui.horizontal(|ui| {
                    let _ = ui.button(format!("{} Arcade", ic::PLAY));
                    let _ = ui.button(format!("{} Training", ic::TARGET));
                    let _ = ui.button(format!("{} Quit", ic::POWER));
                });
            });
    }
}

fn main() {
    let max_frames = std::env::var("UME_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    let demo = Demo {
        ready: false,
        frame: 0,
        t: 0.0,
        volume: 0.7,
        glass: true,
    };
    match run(
        Config {
            title: "egui over quad".into(),
            max_frames,
            ..Default::default()
        },
        demo,
    ) {
        Ok(n) => println!("frames: {n}"),
        Err(e) => eprintln!("{e}"),
    }
}
