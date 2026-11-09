use super::*;
use crate::i18n::tr;

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}
fn ease_back(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0) - 1.0;
    1.0 + x * x * (2.7 * x + 1.7)
}
fn tint(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
fn dark(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

impl App {
    fn versus_tags(&self) -> [&'static str; 2] {
        match self.mode {
            Mode::Versus => ["P1", "P2"],
            Mode::Training => ["P1", tr("BONECO", "DUMMY")],
            Mode::Watch => ["CPU", "CPU"],
            _ => ["P1", "CPU"],
        }
    }

    fn glass(&self, x: f32, y: f32, w: f32, h: f32, accent: Color, alpha: f32) {
        self.ui
            .rect(x, y, w, h, Color::new(0.04, 0.03, 0.1, 0.55 * alpha));
        self.ui
            .rect(x, y, w, h * 0.5, Color::new(1.0, 1.0, 1.0, 0.09 * alpha));
        self.ui
            .rect(x, y, w, 2.0, Color::new(1.0, 1.0, 1.0, 0.55 * alpha));
        self.ui
            .rect(x, y + h - 2.0, w, 2.0, tint(accent, 0.9 * alpha));
    }

    fn versus_team_strip(&mut self, side: usize, x0: f32, y: f32, t: f32) {
        let Some(team) = self.team.clone() else {
            return;
        };
        let members = team.members[side].clone();
        let (card, gap) = (64.0, 6.0);
        let n = members.len();
        for (i, &reg_i) in members.iter().enumerate() {
            let k = ease_out((t - 0.55 - i as f32 * 0.07) / 0.3);
            if k <= 0.0 {
                continue;
            }
            let slide = (1.0 - k) * 60.0 * if side == 0 { -1.0 } else { 1.0 };
            let x = if side == 0 {
                x0 + i as f32 * (card + gap)
            } else {
                x0 - card - (n - 1 - i) as f32 * (card + gap)
            } + slide;
            let (active, down) = (
                team.idx[side] == i || team.simul && i < 2,
                team.ko[side].get(i).copied().unwrap_or(false),
            );
            let col = rgb(self.reg.chars[reg_i].color);
            self.glass(
                x,
                y,
                card,
                card,
                if active { WHITE } else { col },
                k * if down { 0.5 } else { 1.0 },
            );
            if let Some(d) = self.reg.chars[reg_i].peek(1) {
                self.draw_char_fit(
                    &d,
                    0,
                    self.preview_t,
                    x + card / 2.0,
                    y + card - 4.0,
                    card - 8.0,
                    card - 10.0,
                    side == 1,
                );
            }
            let short: String = self.reg.chars[reg_i]
                .name
                .split_whitespace()
                .next()
                .unwrap_or("")
                .chars()
                .take(7)
                .collect();
            self.ui.text(
                &short.to_uppercase(),
                x + card / 2.0,
                y + 12.0,
                11.0,
                if active { hex("#ffcc33") } else { WHITE },
                Align::Center,
                true,
            );
            if active {
                self.ui.rect_lines(
                    x - 2.0,
                    y - 2.0,
                    card + 4.0,
                    card + 4.0,
                    2.0,
                    hex("#ffcc33"),
                );
            }
        }
    }

    pub(super) fn draw_versus(&mut self, t: f32) {
        let (a, b) = (self.sel.0, self.sel.1);
        let (ca, cb) = (rgb(self.reg.chars[a].color), rgb(self.reg.chars[b].color));
        let k = ease_out(t / 0.5);
        let (lo, ro) = (-(1.0 - k) * 760.0, (1.0 - k) * 760.0);
        let (split, lean) = (640.0, 110.0);
        clear_background(hex("#05030c"));

        self.ui.quad(
            [
                vec2(lo - 40.0, 0.0),
                vec2(lo + split + lean / 2.0, 0.0),
                vec2(lo + split - lean / 2.0, VH),
                vec2(lo - 40.0, VH),
            ],
            dark(ca, 0.5),
        );
        self.ui.quad(
            [
                vec2(lo - 40.0, 0.0),
                vec2(lo + split - 160.0, 0.0),
                vec2(lo + split - 160.0 - lean, VH),
                vec2(lo - 40.0, VH),
            ],
            tint(ca, 0.28),
        );
        self.ui.quad(
            [
                vec2(ro + split + lean / 2.0, 0.0),
                vec2(ro + VW + 40.0, 0.0),
                vec2(ro + VW + 40.0, VH),
                vec2(ro + split - lean / 2.0, VH),
            ],
            dark(cb, 0.5),
        );
        self.ui.quad(
            [
                vec2(ro + split + 160.0 + lean, 0.0),
                vec2(ro + VW + 40.0, 0.0),
                vec2(ro + VW + 40.0, VH),
                vec2(ro + split + 160.0, VH),
            ],
            tint(cb, 0.28),
        );

        let drift = (self.ui.t * 30.0) % 80.0;
        for i in 0..12 {
            let x = i as f32 * 80.0 - drift;
            self.ui.quad(
                [
                    vec2(x, 0.0),
                    vec2(x + 3.0, 0.0),
                    vec2(x - lean + 3.0, VH),
                    vec2(x - lean, VH),
                ],
                Color::new(1.0, 1.0, 1.0, 0.03),
            );
        }

        let wm = (t - 0.2).max(0.0).min(1.0);
        let first = |i: usize| {
            self.reg.chars[i]
                .name
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_uppercase()
        };
        let (name_a, name_b) = (first(a), first(b));
        self.ui.text(
            &name_a,
            24.0 + lo * 0.4 + (1.0 - ease_out(wm)) * -200.0,
            270.0,
            190.0,
            Color::new(1.0, 1.0, 1.0, 0.07),
            Align::Left,
            true,
        );
        self.ui.text(
            &name_b,
            VW - 24.0 + ro * 0.4 + (1.0 - ease_out(wm)) * 200.0,
            500.0,
            190.0,
            Color::new(1.0, 1.0, 1.0, 0.07),
            Align::Right,
            true,
        );

        crate::vfx::speed_lines(
            (screen_width() / 2.0, screen_height() * 0.46),
            0.10 + 0.07 * (self.ui.t * 3.0).sin().abs(),
            [1.0, 0.8, 0.5],
            (self.ui.t * 12.0) as u32,
            36,
        );

        let pt = self.preview_t;
        let bob = (self.ui.t * 1.6).sin() * 3.0;
        if let Some(d) = self.reg.chars[a].peek(1) {
            self.draw_char_fit(
                &d,
                0,
                pt,
                330.0 + lo * 1.25,
                596.0 + bob,
                520.0,
                520.0,
                false,
            );
        }
        if let Some(d) = self.reg.chars[b]
            .peek(if a == b { 2 } else { 1 })
            .or_else(|| {
                self.reg.chars[b]
                    .load_preview(if a == b { 2 } else { 1 })
                    .ok()
                    .filter(|_| !self.reg.chars[b].mugen)
            })
        {
            let b_anim = if self.boss_idx() == Some(b) && self.mode == Mode::Arcade {
                let c = self.arcade.as_ref().map(|(l, i)| l[..=*i].iter().filter(|&&x| x == b).count()).unwrap_or(0);
                if c >= 3 { -2000 } else if c == 2 { -1000 } else { 0 }
            } else { 0 };
            self.draw_char_fit(&d, b_anim, pt, 950.0 + ro * 1.25, 596.0 + bob, 520.0, 520.0, true);
        }

        let line = 0.5 + 0.5 * (self.ui.t * 5.0).sin();
        let seam = [
            vec2(split + lean / 2.0 - 5.0, 0.0),
            vec2(split + lean / 2.0 + 5.0, 0.0),
            vec2(split - lean / 2.0 + 5.0, VH),
            vec2(split - lean / 2.0 - 5.0, VH),
        ];
        self.ui
            .quad(seam, Color::new(1.0, 0.92, 0.7, 0.55 + 0.25 * line));

        let tags = self.versus_tags();
        let pk = ease_out((t - 0.3) / 0.35);
        for side in 0..2 {
            let (id, col) = if side == 0 { (a, ca) } else { (b, cb) };
            let w = 540.0;
            let x = if side == 0 {
                40.0 - (1.0 - pk) * 600.0
            } else {
                VW - 40.0 - w + (1.0 - pk) * 600.0
            };
            let y = 590.0;
            self.glass(x, y, w, 84.0, col, pk);
            self.ui
                .rect(if side == 0 { x } else { x + w - 8.0 }, y, 8.0, 84.0, col);
            let (tx, al) = if side == 0 {
                (x + 28.0, Align::Left)
            } else {
                (x + w - 28.0, Align::Right)
            };
            self.ui.text_bold(
                &self.reg.chars[id].name.to_uppercase(),
                tx,
                y + 50.0,
                42.0,
                WHITE,
                BLACK,
                al,
            );
            self.ui.text(
                &self.reg.chars[id].series,
                tx,
                y + 74.0,
                18.0,
                hex("#d8d0f0"),
                al,
                false,
            );
            let chip_x = if side == 0 { x + w - 90.0 } else { x + 20.0 };
            self.ui.rect(chip_x, y + 56.0, 70.0, 22.0, tint(col, 0.9));
            self.ui.text(
                tags[side],
                chip_x + 35.0,
                y + 73.0,
                16.0,
                WHITE,
                Align::Center,
                true,
            );
        }
        if self.team.is_some() {
            self.versus_team_strip(0, 40.0, 88.0, t);
            self.versus_team_strip(1, VW - 40.0, 88.0, t);
        }

        let cx = 640.0;
        let cy = 318.0;
        let ek = ease_back((t - 0.35) / 0.35);
        if ek > 0.0 {
            let pulse = 1.0 + 0.035 * (self.ui.t * 6.0).sin();
            let sc = ek * pulse;
            glow(
                self.ui.x(cx),
                self.ui.y(cy),
                250.0 * sc * self.ui.s,
                Color::new(1.0, 0.5, 0.1, 0.7 * ek.min(1.0)),
            );
            let spin = self.ui.t * 0.9;
            for i in 0..14 {
                let ang = spin + i as f32 * std::f32::consts::TAU / 14.0;
                let (r0, r1, half) = (
                    96.0 * sc,
                    (150.0 + if i % 2 == 0 { 34.0 } else { 0.0 }) * sc,
                    0.09,
                );
                let p = |r: f32, da: f32| {
                    vec2(
                        self.ui.x(cx + (ang + da).cos() * r),
                        self.ui.y(cy + (ang + da).sin() * r),
                    )
                };
                draw_triangle(
                    p(r0, -half),
                    p(r0, half),
                    p(r1, 0.0),
                    Color::new(1.0, 0.82, 0.3, 0.85 * ek.min(1.0)),
                );
            }
            draw_poly(
                self.ui.x(cx),
                self.ui.y(cy),
                48,
                100.0 * sc * self.ui.s,
                0.0,
                Color::new(0.1, 0.02, 0.06, 0.78 * ek.min(1.0)),
            );
            draw_circle_lines(
                self.ui.x(cx),
                self.ui.y(cy),
                100.0 * sc * self.ui.s,
                4.0 * self.ui.s,
                hex("#ffcc33"),
            );
            self.ui.text_bold(
                "VS",
                cx,
                cy + 46.0 * sc,
                132.0 * sc,
                hex("#ffe08a"),
                hex("#5a0a10"),
                Align::Center,
            );
        }

        let tk = ease_out((t - 0.15) / 0.3);
        self.ui.gradient_v(
            0.0,
            0.0,
            VW,
            96.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        let stage_name = self.reg.stages[self.sel.2].name.clone();
        self.glass(
            VW / 2.0 - 230.0,
            18.0 - (1.0 - tk) * 80.0,
            460.0,
            52.0,
            hex("#ffcc33"),
            tk,
        );
        let sub = if let Some(tm) = &self.team {
            if tm.simul {
                format!(
                    "{} · {}x{}",
                    tr("EQUIPE PARALELO", "TEAM SIMUL"),
                    tm.members[0].len(),
                    tm.members[1].len()
                )
            } else {
                format!("{} · {}º vs {}º", tr("EQUIPE SEQUÊNCIA", "TEAM TURNS"), tm.idx[0] + 1, tm.idx[1] + 1)
            }
        } else if let Some((l, i)) = &self.arcade {
            let bcnt = if self.boss_idx().is_some_and(|b| l.get(*i) == Some(&b)) { l[..=*i].iter().filter(|&&x| Some(x) == self.boss_idx()).count() } else { 0 };
            if bcnt >= 3 {
                tr("ARCADE · CLÍMAX SUPREMO: ECLIPSE PRIMORDIAL", "ARCADE · SUPREME CLIMAX: PRIMORDIAL ECLIPSE").to_string()
            } else if bcnt == 2 {
                tr("ARCADE · DESAFIO FINAL: ECLIPSE ABSOLUTO", "ARCADE · FINAL CHALLENGE: ABSOLUTE ECLIPSE").to_string()
            } else {
                format!("ARCADE · {} {}/{}", tr("LUTA", "FIGHT"), i + 1, l.len())
            }
        } else {
            self.mode.label().to_string()
        };
        self.ui.text(
            &stage_name.to_uppercase(),
            VW / 2.0,
            42.0 - (1.0 - tk) * 80.0,
            24.0,
            WHITE,
            Align::Center,
            true,
        );
        self.ui.text(
            &sub,
            VW / 2.0,
            62.0 - (1.0 - tk) * 80.0,
            15.0,
            hex("#d8d0f0"),
            Align::Center,
            false,
        );

        let flash =
            (1.0 - ((t - 0.45).abs() / 0.12)).clamp(0.0, 1.0) * self.g.fx_gain.min(1.0) * 0.55;
        if flash > 0.01 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(1.0, 0.95, 0.85, flash),
            );
        }
    }
}
