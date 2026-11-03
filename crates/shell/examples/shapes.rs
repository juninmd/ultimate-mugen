use ume_render::Color;
use ume_shell::{run, Config, Frame, Game, Key};

struct Demo { x: f32 }

impl Game for Demo {
    fn frame(&mut self, f: &mut Frame) {
        if f.input.is_pressed(Key::Escape) { f.quit = true; }
        if f.input.is_down(Key::ArrowRight) { self.x += 200.0 * f.dt; }
        if f.input.is_down(Key::ArrowLeft) { self.x -= 200.0 * f.dt; }
        f.clear = [0.08, 0.09, 0.14, 1.0];
        let [w, h] = f.size;
        f.canvas.rect(0.0, h * 0.75, w, h * 0.25, Color::rgba(40, 60, 90, 255));
        f.canvas.circle([w * 0.5 + self.x, h * 0.5], h * 0.1, Color::rgba(230, 90, 40, 255));
        f.canvas.line([0.0, h * 0.75], [w, h * 0.75], 4.0, Color::rgba(250, 250, 250, 255));
        f.canvas.rect_lines(20.0, 20.0, 200.0, 24.0, 2.0, Color::WHITE);
        f.canvas.rect(22.0, 22.0, 196.0 * (0.5 + 0.5 * (f.time as f32).sin()), 20.0, Color::rgba(240, 200, 40, 255));
    }
}

fn main() {
    let max_frames = std::env::var("UME_FRAMES").ok().and_then(|v| v.parse().ok());
    match run(Config { title: "ume-shell shapes".into(), max_frames, ..Default::default() }, Demo { x: 0.0 }) {
        Ok(n) => println!("frames: {n}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}
