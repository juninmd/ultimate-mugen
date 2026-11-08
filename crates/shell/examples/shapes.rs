use ume_render::Color;
use ume_shell::text::Font;
use ume_shell::{run, Config, Frame, Game, Key};

struct Demo {
    x: f32,
    title: Font,
    body: Font,
}

impl Game for Demo {
    fn frame(&mut self, f: &mut Frame) {
        if f.input.is_pressed(Key::Escape) {
            f.quit = true;
        }
        if f.input.is_down(Key::ArrowRight) {
            self.x += 200.0 * f.dt;
        }
        if f.input.is_down(Key::ArrowLeft) {
            self.x -= 200.0 * f.dt;
        }
        f.clear = [0.08, 0.09, 0.14, 1.0];
        if std::env::var_os("UME_POST").is_some() {
            f.post = Some(ume_shell::PostParams {
                bloom: 0.9,
                vignette: 0.5,
                chroma: 0.6,
                grain: 0.035,
                contrast: 1.07,
                time: (f.time * 60.0).floor() as f32 % 97.0,
                ..Default::default()
            });
        }
        let [w, h] = f.size;
        f.canvas
            .rect(0.0, h * 0.75, w, h * 0.25, Color::rgba(40, 60, 90, 255));
        f.canvas.circle(
            [w * 0.5 + self.x, h * 0.5],
            h * 0.1,
            Color::rgba(230, 90, 40, 255),
        );
        f.canvas.line(
            [0.0, h * 0.75],
            [w, h * 0.75],
            4.0,
            Color::rgba(250, 250, 250, 255),
        );
        f.canvas
            .rect_lines(20.0, 20.0, 200.0, 24.0, 2.0, Color::WHITE);
        f.canvas.rect(
            22.0,
            22.0,
            196.0 * (0.5 + 0.5 * (f.time as f32).sin()),
            20.0,
            Color::rgba(240, 200, 40, 255),
        );
        let px = h * 0.09;
        let [tw, _] = self.title.measure("ULTIMATE MUGEN", px);
        self.title.draw(
            f.gpu,
            f.canvas,
            "ULTIMATE MUGEN",
            (w - tw) * 0.5,
            h * 0.3,
            px,
            Color::rgba(255, 220, 90, 255),
        );
        self.body.draw(
            f.gpu,
            f.canvas,
            "wgpu text: glyphs rasterized once into the shared atlas",
            24.0,
            h - 30.0,
            h * 0.04,
            Color::WHITE,
        );
    }
}

fn main() {
    let max_frames = std::env::var("UME_FRAMES")
        .ok()
        .and_then(|v| v.parse().ok());
    match run(
        Config {
            title: "ume-shell shapes".into(),
            max_frames,
            ..Default::default()
        },
        Demo {
            x: 0.0,
            title: Font::from_bytes(include_bytes!("../../../assets/fonts/RussoOne.ttf")).unwrap(),
            body: Font::from_bytes(include_bytes!("../../../assets/fonts/Rajdhani.ttf")).unwrap(),
        },
    ) {
        Ok(n) => println!("frames: {n}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
