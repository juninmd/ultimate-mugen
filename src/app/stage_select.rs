use super::*;
use std::cell::Cell;

mod cursor;
mod filter;
mod footer;

const COLS: usize = 4;
const TILE_W: f32 = 290.0;
const TILE_H: f32 = 134.0;
const TOP: f32 = 88.0;
const PREVIEW_DWELL: f32 = 0.12;
const PREVIEW_KEEP_ROWS: usize = 6;

thread_local! {
    static RESTED: Cell<(usize, f32)> = const { Cell::new((usize::MAX, 0.0)) };
    static TRIMMED_ROW: Cell<usize> = const { Cell::new(usize::MAX) };
}

impl App {
    pub(crate) fn open_stages(&mut self, cur: usize) {
        self.set_stage_filter(0, "");
        self.screen = Screen::Stages { cur };
    }

    pub(super) fn stages(&mut self) {
        let Screen::Stages { mut cur } = self.screen else {
            return;
        };
        let typing = self.stage_typing();
        self.ui.background();
        self.ui.text(
            crate::i18n::t("select.choose_stage"),
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.stage_header();
        let view = self.stage_view();
        let n = view.len();
        let total_rows = n.div_ceil(COLS);
        let mut pos = view
            .iter()
            .position(|&v| v == cur)
            .unwrap_or(usize::from(n > 1));
        let old = pos;
        if !typing {
            if self.c.pressed(nav::RIGHT) {
                pos = (pos + 1) % n;
            }
            if self.c.pressed(nav::LEFT) {
                pos = (pos + n - 1) % n;
            }
            if self.c.pressed(nav::DOWN) {
                pos = (pos + COLS).min(n - 1);
            }
            if self.c.pressed(nav::UP) {
                pos = pos.saturating_sub(COLS);
            }
            if is_key_pressed(KeyCode::PageDown) {
                pos = (pos + COLS * VIEW_ROWS).min(n - 1);
            }
            if is_key_pressed(KeyCode::PageUp) {
                pos = pos.saturating_sub(COLS * VIEW_ROWS);
            }
            if is_key_pressed(KeyCode::Home) {
                pos = 0;
            }
            if is_key_pressed(KeyCode::End) {
                pos = n - 1;
            }
        }
        let wheel = mouse_wheel().1;
        if wheel > 0.0 {
            pos = pos.saturating_sub(COLS);
        } else if wheel < 0.0 {
            pos = (pos + COLS).min(n - 1);
        }
        if pos != old {
            self.audio.sfx("select");
        }
        let crow = pos / COLS;
        if crow < self.stage_row {
            self.stage_row = crow;
        }
        if crow >= self.stage_row + VIEW_ROWS {
            self.stage_row = crow + 1 - VIEW_ROWS;
        }
        self.stage_row = self.stage_row.min(total_rows.saturating_sub(VIEW_ROWS));
        let mut ok = !typing && self.c.pressed(nav::OK | nav::START);
        let rested = RESTED.with(|r| {
            let (at, t) = r.get();
            let t = if at == pos { t + get_frame_time() } else { 0.0 };
            r.set((pos, t));
            t >= PREVIEW_DWELL
        });
        let first = self.stage_row * COLS;
        let last = ((self.stage_row + VIEW_ROWS) * COLS).min(n);
        let mut requested = false;
        for k in first..last {
            let i = view[k];
            let (x, y) = (
                40.0 + (k % COLS) as f32 * (TILE_W + 13.0),
                TOP + ((k / COLS) - self.stage_row) as f32 * (TILE_H + 14.0),
            );
            if self.clicked_rect(x, y, TILE_W, TILE_H) {
                if pos == k {
                    ok = true;
                }
                pos = k;
            }
            let (name, th, mugen) = if i == 0 {
                (
                    crate::i18n::t("select.random").to_string(),
                    [[255, 47, 208], [34, 232, 255], [255, 226, 58]],
                    false,
                )
            } else {
                let s = &self.reg.stages[i - 1];
                (s.name.clone(), s.thumb, s.mugen)
            };
            self.ui
                .gradient_v(x, y, TILE_W, TILE_H * 0.5, rgb(th[0]), rgb(th[1]));
            self.ui.gradient_v(
                x,
                y + TILE_H * 0.5,
                TILE_W,
                TILE_H * 0.5,
                rgb(th[1]),
                rgb(th[2]),
            );
            let preview = i.checked_sub(1).and_then(|index| {
                let started = self.reg.stage_preview_started(index);
                if !started && (requested || !rested) {
                    return None;
                }
                requested |= !started;
                self.reg.stage_preview(index)
            });
            if let Some(image) = preview {
                let texture = self.g.texture(&image, true);
                draw_texture_ex(
                    &texture,
                    self.ui.x(x),
                    self.ui.y(y),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_W * self.ui.s, TILE_H * self.ui.s)),
                        source: Some(Rect::new(
                            0.0,
                            (image.h as f32 - image.w as f32 * TILE_H / TILE_W) * cursor::pan(pos == k, self.ui.t),
                            image.w as f32,
                            image.w as f32 * TILE_H / TILE_W,
                        )),
                        ..Default::default()
                    },
                );
            }
            self.ui.rect(
                x,
                y + TILE_H - 30.0,
                TILE_W,
                30.0,
                Color::new(0.0, 0.0, 0.0, 0.6),
            );
            let label = self.fit_label(&name, 16.0, TILE_W - 20.0);
            self.ui.text(
                &label,
                x + 10.0,
                y + TILE_H - 9.0,
                16.0,
                WHITE,
                Align::Left,
                true,
            );
            if mugen {
                self.ui
                    .rect(x + TILE_W - 60.0, y + 6.0, 54.0, 16.0, hex("#22e8ff"));
                self.ui.text(
                    "MUGEN",
                    x + TILE_W - 33.0,
                    y + 19.0,
                    12.0,
                    BLACK,
                    Align::Center,
                    true,
                );
            }
        }
        if (first..last).contains(&pos) {
            let x = 40.0 + (pos % COLS) as f32 * (TILE_W + 13.0);
            self.draw_stage_cursor(x, TOP + ((pos / COLS) - self.stage_row) as f32 * (TILE_H + 14.0));
        }
        self.trim_far_previews(&view);
        if total_rows > VIEW_ROWS {
            let bh = VIEW_ROWS as f32 * (TILE_H + 14.0) - 14.0;
            self.ui
                .rect(VW - 22.0, TOP, 6.0, bh, Color::new(1.0, 1.0, 1.0, 0.1));
            let th = (bh * VIEW_ROWS as f32 / total_rows as f32).max(18.0);
            let ty = TOP + (bh - th) * self.stage_row as f32 / (total_rows - VIEW_ROWS) as f32;
            self.ui.rect(VW - 22.0, ty, 6.0, th, hex("#ff4d6d"));
        }
        self.stage_footer(pos, n, total_rows);
        cur = view[pos];
        if ok && n > 1 {
            let pool: Vec<usize> = if view.len() > 1 {
                view[1..].iter().map(|v| v - 1).collect()
            } else {
                (0..self.reg.stages.len()).collect()
            };
            self.sel.2 = if cur == 0 {
                pool[(get_time() * 7.0) as usize % pool.len()]
            } else {
                cur - 1
            };
            self.go(Screen::Vs { t: 0.0, step: 0 });
            return;
        }
        if !typing && self.c.pressed(nav::BACK) {
            self.screen = Screen::Select {
                cur: [self.sel.0, self.sel.1],
                picking: 0,
                picked: [None, None],
            };
            return;
        }
        self.screen = Screen::Stages { cur };
    }
}
