use crate::app::{App, Screen};
use crate::gfx::*;

const FRAMES_PER_STEP: usize = 9;

pub(crate) async fn run(app: &mut App, dir: &str) {
    let dir = format!("{dir}/menu-motion");
    let _ = std::fs::create_dir_all(&dir);
    let mut shot = 0;
    for cur in [0usize, 1, 2, 3, 2, 1, 0] {
        app.screen = Screen::Menu { cur };
        for _ in 0..FRAMES_PER_STEP {
            app.trans = 0.0;
            app.frame().await;
            get_screen_data().export_png(&format!("{dir}/{shot:03}.png"));
            shot += 1;
            next_frame().await;
        }
    }
    println!("shot menu-motion ({shot} frames)");
}
