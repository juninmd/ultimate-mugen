use crate::gfx::*;
use crate::ui::{hex, Align};
use std::cell::Cell;

thread_local! {
    static BEST: Cell<(i32, i32)> = Cell::new((0, 0));
}

pub(super) fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    p: &P,
    text: &T,
    s: f32,
    w: f32,
    hits: i32,
    dmg: f32,
) {
    BEST.with(|b| {
        let (bh, bd) = b.get();
        if hits >= 2 && (hits > bh || (hits == bh && dmg as i32 > bd)) {
            b.set((hits, dmg as i32));
        }
    });
    let (best_hits, best_dmg) = BEST.with(|b| b.get());
    let tape = crate::session::dummy_tape::len();
    let recording = crate::session::dummy_tape::is_recording();
    if best_hits == 0 && tape == 0 && !recording {
        return;
    }
    let (x, y) = (w - 68.0, 50.0);
    let corner = p(x, y);
    draw_rectangle(
        corner.x,
        corner.y,
        62.0 * s,
        20.0 * s,
        Color::new(0.02, 0.02, 0.06, 0.55),
    );
    if best_hits > 0 {
        text(
            &format!("BEST {best_hits}H · {best_dmg}"),
            x + 4.0,
            y + 6.0,
            5.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
    }
    if recording || tape > 0 {
        let label = if recording { "REC" } else { "TAPE" };
        let color = if recording {
            hex("#ff4757")
        } else {
            hex("#2ed573")
        };
        text(
            &format!("{label} {:.1}s", tape as f32 / 60.0),
            x + 4.0,
            y + 13.0,
            5.0,
            color,
            Align::Left,
            true,
        );
    }
}
