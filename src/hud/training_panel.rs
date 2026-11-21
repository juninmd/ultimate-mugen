use crate::gfx::*;
use crate::ui::{hex, Align};
use std::cell::Cell;

thread_local! {
    static BEST: Cell<(u32, i32, i32)> = Cell::new((0, 0, 0));
}

pub(super) fn reset() {
    BEST.with(|b| b.set((0, 0, 0)));
}

pub(super) fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    p: &P,
    text: &T,
    s: f32,
    w: f32,
    owner: u32,
    hits: i32,
    dmg: f32,
) {
    let (mut best_owner, mut best_hits, mut best_dmg) = BEST.with(|b| b.get());
    if best_owner != owner {
        best_owner = owner;
        best_hits = 0;
        best_dmg = 0;
    }
    if hits >= 2 && (hits > best_hits || (hits == best_hits && dmg as i32 > best_dmg)) {
        best_hits = hits;
        best_dmg = dmg as i32;
    }
    BEST.with(|b| b.set((best_owner, best_hits, best_dmg)));
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
