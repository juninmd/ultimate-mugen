use crate::gfx::*;
use crate::session::debug_ctrl::{self, DebugLayers};
use crate::ui::{hex, Align};
use ume_engine::matchs::Match;

const TAB_NAMES: [&str; 4] = ["OVERVIEW", "VARS & MAPS", "CLIPBOARD", "CONSOLE"];

pub(super) fn draw_toolbar<
    P: Fn(f32, f32) -> Vec2,
    T: Fn(&str, f32, f32, f32, Color, Align, bool),
>(
    p: &P,
    text: &T,
    s: f32,
    w: f32,
    m: &Match,
) {
    let top_y = 2.5;
    let c = p(4.0, top_y);
    draw_rectangle(
        c.x,
        c.y,
        (w - 8.0) * s,
        10.5 * s,
        Color::new(0.03, 0.04, 0.1, 0.82),
    );
    draw_rectangle_lines(
        c.x,
        c.y,
        (w - 8.0) * s,
        10.5 * s,
        1.0,
        Color::new(0.35, 0.85, 1.0, 0.55),
    );
    let tab = debug_ctrl::tab();
    let mode_tag = if debug_ctrl::frozen() {
        "FROZEN"
    } else {
        "LIVE"
    };
    let lyr = match debug_ctrl::layers() {
        DebugLayers::All => "ALL",
        DebugLayers::BoxesOnly => "BOXES",
        DebugLayers::HudOnly => "HUD",
    };
    let helpers = m
        .fighters
        .iter()
        .filter(|f| !f.destroyed && f.is_helper)
        .count();
    let left_txt = format!(
        "DBG F:{} {} {:.2}x | CAM ({:.0},{:.0}) z{:.2} | H:{} P:{} E:{}",
        m.game_time,
        mode_tag,
        debug_ctrl::speed_mul(),
        m.camera.x,
        m.camera.y,
        m.zoom,
        helpers,
        m.projectiles.len(),
        m.explods.len()
    );
    text(
        &left_txt,
        7.0,
        top_y + 7.2,
        4.1,
        hex("#7ff0ff"),
        Align::Left,
        false,
    );
    let hint = format!(
        "[F4]:{}  [F5]:PAUSE  [F6]:+1F  [F7]:SPD  [F8]:{}",
        TAB_NAMES[tab as usize], lyr
    );
    text(
        &hint,
        w - 7.0,
        top_y + 7.2,
        4.0,
        hex("#ffd66b"),
        Align::Right,
        false,
    );
}

pub(super) fn draw_console_drawer<
    P: Fn(f32, f32) -> Vec2,
    T: Fn(&str, f32, f32, f32, Color, Align, bool),
>(
    p: &P,
    text: &T,
    s: f32,
    w: f32,
    h: f32,
    m: &Match,
    tab: debug_ctrl::DebugTab,
) {
    let lines: Vec<String> = m.console.lines().rev().take(5).cloned().collect();
    if tab != debug_ctrl::DebugTab::Console && lines.is_empty() && m.warnings.is_empty() {
        return;
    }
    let show: Vec<String> = if lines.is_empty() {
        m.warnings
            .iter()
            .rev()
            .take(4)
            .map(|w| format!("! {w}"))
            .collect()
    } else {
        lines
    };
    if show.is_empty() && tab != debug_ctrl::DebugTab::Console {
        return;
    }
    let rows = show.len().max(1) + 1;
    let bh = rows as f32 * 6.2 + 4.0;
    let y = h - bh - 4.0;
    let x = 6.0;
    let bw = w - 12.0;
    let c = p(x, y);
    draw_rectangle(c.x, c.y, bw * s, bh * s, Color::new(0.02, 0.02, 0.06, 0.78));
    draw_rectangle_lines(
        c.x,
        c.y,
        bw * s,
        bh * s,
        1.0,
        Color::new(0.45, 0.75, 1.0, 0.45),
    );
    text(
        "CONSOLE / ENGINE LOG",
        x + 4.0,
        y + 6.0,
        4.1,
        hex("#ffd66b"),
        Align::Left,
        false,
    );
    if show.is_empty() {
        text(
            "(no console messages)",
            x + 4.0,
            y + 12.2,
            4.0,
            hex("#9aa6c4"),
            Align::Left,
            false,
        );
    } else {
        for (i, line) in show.iter().rev().enumerate() {
            text(
                line,
                x + 4.0,
                y + 12.2 + i as f32 * 6.2,
                4.0,
                hex("#d6ecff"),
                Align::Left,
                false,
            );
        }
    }
}
