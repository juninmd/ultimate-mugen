use super::snap;
use crate::app::App;
use crate::gfx::*;
use crate::ui::*;

const COLUMNS: usize = 5;
const FLAG_W: f32 = 180.0;
const CELL: (f32, f32) = (240.0, 220.0);
const ORIGIN: (f32, f32) = (70.0, 50.0);

pub(super) async fn run(app: &mut App, dir: &str) {
    snap(app, dir, "flags", |app| {
        app.ui.begin();
        clear_background(Color::new(0.07, 0.06, 0.12, 1.0));
        for (n, code) in crate::flag::catalog::codes().enumerate() {
            let x = ORIGIN.0 + (n % COLUMNS) as f32 * CELL.0;
            let y = ORIGIN.1 + (n / COLUMNS) as f32 * CELL.1;
            app.ui.flag(code, x, y, FLAG_W, 1.0);
            app.ui.flag(code, x, y + 136.0, 20.0, 1.0);
            app.ui.text(
                &code.to_uppercase(),
                x + 30.0,
                y + 150.0,
                16.0,
                WHITE,
                Align::Left,
                true,
            );
        }
    })
    .await;
}
