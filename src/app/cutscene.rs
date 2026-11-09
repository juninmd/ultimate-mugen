use super::*;
use crate::poly;

#[path = "cutscene/battle.rs"]
mod battle;
#[path = "cutscene/climax.rs"]
mod climax;
#[path = "cutscene/endings.rs"]
mod endings;
#[path = "cutscene/story.rs"]
mod story;

pub struct Page {
    pub who: u8,
    pub text: String,
}
pub(self) fn pg(who: u8, text: impl Into<String>) -> Page {
    Page { who, text: text.into() }
}

pub fn script(kind: u8, hero: &str, hero_quote: &str, boss: &str, boss_line: &str) -> Vec<Page> {
    script_for(kind, hero, hero, hero_quote, boss, boss_line)
}

pub fn script_for(kind: u8, hero_id: &str, hero: &str, hero_quote: &str, boss: &str, boss_line: &str) -> Vec<Page> {
    let clean_boss = boss.strip_prefix("umf:").unwrap_or(boss).to_lowercase();
    match kind {
        0 => story::opening_script(hero_id, hero),
        1 => {
            if clean_boss.contains("garan") || clean_boss.contains("valdric") {
                let boss_key = if clean_boss.contains("garan") { "garan" } else { "valdric" };
                crate::boss::dialogue::script(1, hero, hero_quote, boss_key, boss, boss_line)
                    .into_iter()
                    .map(|p| Page { who: p.who, text: p.text })
                    .collect()
            } else {
                story::preboss_script(hero_id, boss, boss_line)
            }
        }
        3 => story::awakening_script(hero_id, hero),
        4 => climax::climax_script(hero_id, hero, boss),
        5 => climax::defeat_script(hero_id, hero, boss),
        _ => endings::ending_script(hero_id, hero, hero_quote),
    }
}

impl App {
    fn native_quote(&self, i: usize, boss_line: bool) -> String {
        match &self.reg.chars[i].src {
            crate::registry::Src::Native(nc) => {
                if boss_line { nc.intro.clone().unwrap_or_default() } else { nc.win_quote.clone().unwrap_or_default() }
            }
            _ => String::new(),
        }
    }

    pub(super) fn cutscene(&mut self) {
        let Screen::Cutscene { kind, mut page, mut t } = self.screen else { return; };
        t += get_frame_time();
        let hero = self.sel.0;
        let Some(boss) = self.boss_idx() else { self.cutscene_done(kind); return; };
        let (hero_id, hero_name) = (self.reg.chars[hero].id.clone(), self.reg.chars[hero].name.clone());
        let boss_name = self.reg.chars[boss].name.clone();
        let script = script_for(kind, &hero_id, &hero_name, &self.native_quote(hero, false), &boss_name, &self.native_quote(boss, true));
        if page >= script.len() { self.cutscene_done(kind); return; }
        let cur = &script[page];
        let hc = rgb(self.reg.chars[hero].color);
        let bc = if kind == 4 { hex("#00f0ff") } else if kind == 3 { hex("#ffbe3d") } else { rgb(self.reg.chars[boss].color) };
        let (sky_top, sky_bot) = match kind {
            2 => (alpha(hc, 0.32), hex("#070510")),
            3 => (hex("#24051a"), hex("#06010a")),
            4 => (hex("#04142a"), hex("#020510")),
            5 => (hex("#1a0822"), hex("#020106")),
            _ => (hex("#14061c"), hex("#050208")),
        };
        self.ui.gradient_v(0.0, 0.0, 1280.0, 720.0, sky_top, sky_bot);
        let (mid, s) = (vec2(self.ui.x(640.0), self.ui.y(250.0)), self.ui.s);
        let pulse = 1.0 + 0.03 * (self.ui.t * if kind >= 3 { 2.8 } else { 1.3 }).sin();
        let glow_col = match kind {
            2 => Color::new(hc.r, hc.g, hc.b, 0.045),
            4 => Color::new(0.0, 0.8, 1.0, 0.05),
            3 => Color::new(1.0, 0.55, 0.15, 0.05),
            _ => Color::new(0.8, 0.05, 0.2, 0.035),
        };
        for k in 0..8 { poly::circle(mid, (190.0 + k as f32 * 26.0) * s * pulse, glow_col); }
        if kind == 2 {
            poly::circle(mid, 148.0 * s, Color::new(1.0, 0.88, 0.55, 0.22));
            poly::ring(mid, 154.0 * s, 3.5 * s, alpha(hc, 0.75));
        } else if kind == 4 {
            poly::circle(mid, 160.0 * s, hex("#041830"));
            poly::circle(mid, 146.0 * s, hex("#020612"));
            poly::ring(mid, 154.0 * s, 4.5 * s, Color::new(0.0, 0.95, 1.0, 0.9));
            poly::ring(mid, 180.0 * s * pulse, 2.0 * s, Color::new(1.0, 0.0, 0.35, 0.65));
        } else if kind == 3 {
            poly::circle(mid, 156.0 * s, hex("#3a0518"));
            poly::circle(mid, 144.0 * s, hex("#09010d"));
            poly::ring(mid, 150.0 * s, 4.5 * s, Color::new(1.0, 0.78, 0.22, 0.9));
            poly::ring(mid, 174.0 * s * pulse, 2.0 * s, Color::new(1.0, 0.22, 0.38, 0.65));
        } else if kind != 5 {
            poly::circle(mid, 150.0 * s, hex("#7a0c28"));
            poly::circle(mid + vec2(34.0 * s, -10.0 * s), 140.0 * s, hex("#14061c"));
            poly::ring(mid, 152.0 * s, 3.0 * s, Color::new(1.0, 0.3, 0.3, 0.7));
        }
        for i in 0..7 {
            let f = i as f32;
            let x = ((self.ui.t * (28.0 + f * 6.0) + f * 211.0) % 1500.0) - 110.0;
            let y = 90.0 + f * 31.0 + (self.ui.t * 1.7 + f).sin() * 12.0;
            let flap = (self.ui.t * 9.0 + f * 1.3).sin() * 7.0;
            let c = vec2(self.ui.x(x), self.ui.y(y));
            poly::solid(&[c + vec2(-12.0, flap) * s, c + vec2(0.0, -2.0) * s, c + vec2(12.0, flap) * s, c + vec2(0.0, 4.0) * s], Color::new(0.02, 0.0, 0.04, 0.9));
        }
        self.ui.motes(if kind >= 3 { 65 } else { 40 }, if kind >= 3 { 1.3 } else { 0.8 });
        let fl = match kind {
            2 => alpha(hc, 0.32),
            4 => Color::new(0.0, 0.45, 0.85, 0.42),
            3 => Color::new(0.85, 0.35, 0.1, 0.42),
            _ => Color::new(0.5, 0.05, 0.15, 0.35),
        };
        self.ui.gradient_v(0.0, 560.0, 1280.0, 160.0, Color::new(fl.r, fl.g, fl.b, 0.0), fl);
        let pt = self.preview_t;
        let hd = self.reg.chars[hero].peek(1).or_else(|| self.reg.chars[hero].load_preview(1).ok());
        let bd = if self.reg.chars[boss].mugen { None } else { self.reg.chars[boss].load(1).ok() };
        if kind == 4 || kind == 5 {
            battle::draw_battle_effects(self, t, page, kind == 5, &hd, &bd, hc, bc);
        } else {
            let anim_h = if kind == 2 { 181 } else { 0 };
            let anim_b = match kind {
                2 => 5150,
                3 if page == 0 => 5190,
                3 if page >= 3 => -1195,
                3 => 196,
                1 if cur.who == 2 => 195,
                _ => 0,
            };
            let slide = (1.0 - (t / 0.5).min(1.0)).powi(2) * 220.0;
            let dim = Color::new(0.42, 0.4, 0.48, 1.0);
            self.char_tint = if cur.who == 0 { Color::new(0.84, 0.84, 0.9, 1.0) } else if cur.who == 1 { WHITE } else { dim };
            if let Some(d) = &hd { self.draw_char(d, anim_h, pt, 310.0 - slide, 650.0, 470.0, false); }
            self.char_tint = if cur.who == 0 { Color::new(0.84, 0.84, 0.9, 1.0) } else if cur.who == 2 { WHITE } else { dim };
            if kind != 0 {
                if let Some(d) = &bd { self.draw_char(d, anim_b, pt, 970.0 + slide, 650.0, 500.0, true); }
            }
            self.char_tint = WHITE;
        }
        self.ui.rect(0.0, 0.0, 1280.0, 56.0, BLACK);
        self.ui.rect(0.0, 666.0, 1280.0, 54.0, BLACK);
        let (chap, sub_chap) = story::chapter_title(kind, &hero_name);
        self.ui.text(chap, 94.0, 24.0, 14.0, alpha(bc, 0.9), Align::Left, true);
        self.ui.text(&sub_chap, 94.0, 44.0, 18.0, WHITE, Align::Left, true);
        self.ui.text(&format!("{}/{}", page + 1, script.len()), 1186.0, 38.0, 15.0, hex("#a49cc0"), Align::Right, true);
        let (name, col) = match cur.who {
            1 => (hero_name.to_uppercase(), hc),
            2 if kind == 4 && page >= 4 => (format!("{} · ECLIPSE PRIMORDIAL", boss_name.to_uppercase()), bc),
            2 if kind == 3 && page >= 3 => (format!("{} · ECLIPSE ABSOLUTO", boss_name.to_uppercase()), bc),
            2 => (boss_name.to_uppercase(), bc),
            _ => ("NARRADOR".to_string(), hex("#a49cc0")),
        };
        let box_y = 520.0;
        let panel = |x: f32, y: f32, w: f32, h: f32| -> Vec<Vec2> {
            [(x + 14.0, y), (x + w, y), (x + w - 14.0, y + h), (x, y + h)].iter().map(|&(a, b)| vec2(self.ui.x(a), self.ui.y(b))).collect()
        };
        poly::grad_v(&panel(90.0, box_y, 1100.0, 130.0), Color::new(0.05, 0.03, 0.12, 0.92), Color::new(0.02, 0.01, 0.06, 0.92));
        poly::outline(&panel(90.0, box_y, 1100.0, 130.0), 2.0 * s, alpha(col, 0.9));
        poly::grad_h(&panel(90.0, box_y - 34.0, 440.0, 34.0), alpha(col, 0.95), alpha(col, 0.35));
        self.ui.text(&name, 112.0, box_y - 9.0, 22.0, WHITE, Align::Left, true);
        let shown = ((t * 48.0) as usize).min(cur.text.chars().count());
        let txt: String = cur.text.chars().take(shown).collect();
        let (mut line, mut y) = (String::new(), box_y + 44.0);
        for w in txt.split(' ') {
            if self.ui.measure(&format!("{line} {w}"), 24.0, false) > 1030.0 {
                self.ui.text(&line, 116.0, y, 24.0, WHITE, Align::Left, false);
                y += 32.0;
                line.clear();
            }
            if !line.is_empty() { line.push(' '); }
            line.push_str(w);
        }
        self.ui.text(&line, 116.0, y, 24.0, WHITE, Align::Left, false);
        let done = shown >= cur.text.chars().count();
        if done {
            let a = 0.5 + 0.5 * (self.ui.t * 5.0).sin().abs();
            let c = vec2(self.ui.x(1160.0), self.ui.y(box_y + 108.0));
            poly::solid(&[c + vec2(-7.0, -8.0) * s, c + vec2(8.0, 0.0) * s, c + vec2(-7.0, 8.0) * s], Color::new(1.0, 1.0, 1.0, a));
        }
        let hint = if kind == 2 && page + 1 == script.len() {
            "ENTER voltar ao menu principal · ESC pular"
        } else {
            "ENTER avançar · ESC pular"
        };
        self.ui.text(hint, 1240.0, 702.0, 14.0, Color::new(1.0, 1.0, 1.0, 0.45), Align::Right, false);
        if kind == 0 {
            if let Some(txt) = self.sp.as_ref().and_then(|sp| sp.intro_text((t * 60.0) as i64)) {
                self.ui.text(&txt, VW / 2.0, 96.0, 26.0, hex("#ffcc33"), Align::Center, true);
            }
        }
        let adv = self.c.pressed(nav::OK) || is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        if self.c.pressed(nav::BACK | nav::START) { self.cutscene_done(kind); return; }
        if adv {
            if !done { t = 99.0; } else { page += 1; t = 0.0; self.audio.sfx("select"); }
        }
        if let Screen::Cutscene { .. } = self.screen { self.screen = Screen::Cutscene { kind, page, t }; }
    }

    pub(crate) fn cutscene_done(&mut self, kind: u8) {
        self.screen = match kind {
            0 => Screen::Stages { cur: 1 },
            1 | 3 | 4 => Screen::Vs { t: 0.0, step: 0 },
            5 => Screen::Cutscene { kind: 2, page: 0, t: 0.0 },
            _ => {
                self.replaying = None;
                self.session = None;
                self.team = None;
                self.arcade = None;
                Screen::Menu { cur: 0 }
            }
        };
    }
}
