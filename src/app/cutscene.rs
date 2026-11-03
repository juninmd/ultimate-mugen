//! Story cutscenes (arcade opening, face-off with the final boss, ending): letterboxed dialogue scenes with live character art.
use super::*;
use crate::poly;

/// Who speaks: 0 narrator, 1 the player's fighter, 2 the boss.
pub struct Page { pub who: u8, pub text: String }
fn pg(who: u8, text: impl Into<String>) -> Page { Page { who, text: text.into() } }

/// Script of a cutscene (`kind` 0 = arcade opening, 1 = before the boss, 2 = ending).
pub fn script(kind: u8, hero: &str, hero_quote: &str, boss: &str, boss_line: &str) -> Vec<Page> {
    let first = hero.split_whitespace().next().unwrap_or(hero).to_string();
    match kind {
        0 => vec![
            pg(0, "O Torneio do Eclipse Escarlate reúne os maiores lutadores de todas as eras."),
            pg(0, "Mas diz a lenda que, no fim do caminho, aguarda um adversário que nunca perdeu — e nunca precisou lutar de verdade."),
            pg(1, "Não importa quem me espera no final. Eu vou até o fim."),
        ],
        1 => vec![
            pg(0, "Oito rivais caíram. O salão final se abre sob uma lua vermelha."),
            pg(2, if boss_line.is_empty() { "Você chegou até aqui... exatamente como eu havia previsto." } else { boss_line }),
            pg(1, "Quem é você?"),
            pg(2, format!("{boss}. Aquele que já viu o fim desta luta antes mesmo de ela começar.")),
            pg(1, "Então vai ver você perder também."),
            pg(2, "Que adorável. Mostre-me... o que existe depois do final."),
        ],
        _ => vec![
            pg(2, "Impossível... eu havia previsto cada movimento seu..."),
            pg(1, if hero_quote.is_empty() { "Você previu o que eu faria. Não o quanto eu queria vencer.".to_string() } else { hero_quote.to_string() }),
            pg(0, "A lua vermelha se parte em mil pedaços de luz."),
            pg(0, format!("O torneio chegou ao fim — e uma nova lenda nasce: {first}.")),
        ],
    }
}

impl App {
    fn native_quote(&self, i: usize, boss_line: bool) -> String {
        match &self.reg.chars[i].src {
            crate::registry::Src::Native(nc) => if boss_line { nc.intro.clone().unwrap_or_default() } else { nc.win_quote.clone().unwrap_or_default() },
            _ => String::new(),
        }
    }

    pub(super) fn cutscene(&mut self) {
        let Screen::Cutscene { kind, mut page, mut t } = self.screen else { return };
        let dt = get_frame_time();
        t += dt;
        let hero = self.sel.0;
        let Some(boss) = self.boss_idx() else { self.cutscene_done(kind); return };
        let script = script(kind, &self.reg.chars[hero].name, &self.native_quote(hero, false), &self.reg.chars[boss].name, &self.native_quote(boss, true));
        if page >= script.len() { self.cutscene_done(kind); return; }
        let cur = &script[page];
        let (hc, bc) = (rgb(self.reg.chars[hero].color), rgb(self.reg.chars[boss].color));

        // ---- backdrop: blood-red eclipse over a dark gradient, drifting crows
        self.ui.gradient_v(0.0, 0.0, 1280.0, 720.0, hex("#14061c"), hex("#050208"));
        let mid = vec2(self.ui.x(640.0), self.ui.y(250.0));
        let s = self.ui.s;
        let pulse = 1.0 + 0.02 * (self.ui.t * 1.3).sin();
        for k in 0..8 { poly::circle(mid, (190.0 + k as f32 * 26.0) * s * pulse, Color::new(0.8, 0.05, 0.2, 0.035)); }
        poly::circle(mid, 150.0 * s, hex("#7a0c28"));
        poly::circle(mid + vec2(34.0 * s, -10.0 * s), 140.0 * s, hex("#14061c"));
        poly::ring(mid, 152.0 * s, 3.0 * s, Color::new(1.0, 0.3, 0.3, 0.7));
        for i in 0..7 {
            let f = i as f32;
            let x = ((self.ui.t * (28.0 + f * 6.0) + f * 211.0) % 1500.0) - 110.0;
            let y = 90.0 + f * 31.0 + (self.ui.t * 1.7 + f).sin() * 12.0;
            let flap = (self.ui.t * 9.0 + f * 1.3).sin() * 7.0;
            let c = vec2(self.ui.x(x), self.ui.y(y));
            poly::solid(&[c + vec2(-12.0, flap) * s, c + vec2(0.0, -2.0) * s, c + vec2(12.0, flap) * s, c + vec2(0.0, 4.0) * s], Color::new(0.02, 0.0, 0.04, 0.9));
        }
        self.ui.motes(40, 0.8);
        // floor glow
        self.ui.gradient_v(0.0, 560.0, 1280.0, 160.0, Color::new(0.5, 0.05, 0.15, 0.0), Color::new(0.5, 0.05, 0.15, 0.35));

        // ---- characters (the speaker is lit, the other side dimmed)
        let pt = self.preview_t;
        let hd = self.reg.chars[hero].peek(1).or_else(|| self.reg.chars[hero].load_preview(1).ok());
        let bd = if self.reg.chars[boss].mugen { None } else { self.reg.chars[boss].load(1).ok() };
        let anim_h = if kind == 2 { 181 } else { 0 };
        let anim_b = if kind == 2 { 5150 } else if cur.who == 2 && kind == 1 { 195 } else { 0 };
        let slide = (1.0 - (t / 0.5).min(1.0)).powi(2) * 220.0;
        // the one who is not speaking is shaded (narration: both slightly dimmed)
        let dim = Color::new(0.42, 0.4, 0.48, 1.0);
        let lit = |on: bool| if on { WHITE } else { dim };
        self.char_tint = if cur.who == 0 { Color::new(0.8, 0.8, 0.85, 1.0) } else { lit(cur.who == 1) };
        if let Some(d) = &hd { self.draw_char(d, anim_h, pt, 310.0 - slide, 650.0, 470.0, false); }
        self.char_tint = if cur.who == 0 { Color::new(0.8, 0.8, 0.85, 1.0) } else { lit(cur.who == 2) };
        if kind != 0 { if let Some(d) = &bd { self.draw_char(d, anim_b, pt, 970.0 + slide, 650.0, 500.0, true); } }
        self.char_tint = WHITE;

        // ---- cinematic letterbox + dialogue panel
        self.ui.rect(0.0, 0.0, 1280.0, 54.0, BLACK);
        self.ui.rect(0.0, 666.0, 1280.0, 54.0, BLACK);
        let (name, col) = match cur.who { 1 => (self.reg.chars[hero].name.to_uppercase(), hc), 2 => (self.reg.chars[boss].name.to_uppercase(), bc), _ => ("NARRADOR".to_string(), hex("#a49cc0")) };
        let box_y = 520.0;
        let panel = |x: f32, y: f32, w: f32, h: f32| -> Vec<Vec2> { [(x + 14.0, y), (x + w, y), (x + w - 14.0, y + h), (x, y + h)].iter().map(|&(a, b)| vec2(self.ui.x(a), self.ui.y(b))).collect() };
        poly::grad_v(&panel(90.0, box_y, 1100.0, 130.0), Color::new(0.05, 0.03, 0.12, 0.92), Color::new(0.02, 0.01, 0.06, 0.92));
        poly::outline(&panel(90.0, box_y, 1100.0, 130.0), 2.0 * s, alpha(col, 0.9));
        poly::grad_h(&panel(90.0, box_y - 34.0, 300.0, 34.0), alpha(col, 0.95), alpha(col, 0.35));
        self.ui.text(&name, 112.0, box_y - 9.0, 24.0, WHITE, Align::Left, true);
        // typewriter
        let shown = ((t * 48.0) as usize).min(cur.text.chars().count());
        let txt: String = cur.text.chars().take(shown).collect();
        let (mut line, mut y) = (String::new(), box_y + 44.0);
        for w in txt.split(' ') {
            if self.ui.measure(&format!("{line} {w}"), 24.0, false) > 1030.0 { self.ui.text(&line, 116.0, y, 24.0, WHITE, Align::Left, false); y += 32.0; line.clear(); }
            if !line.is_empty() { line.push(' '); }
            line.push_str(w);
        }
        self.ui.text(&line, 116.0, y, 24.0, WHITE, Align::Left, false);
        let done = shown >= cur.text.chars().count();
        if done { let a = 0.5 + 0.5 * (self.ui.t * 5.0).sin().abs(); { let c = vec2(self.ui.x(1160.0), self.ui.y(box_y + 108.0)); poly::solid(&[c + vec2(-7.0, -8.0) * s, c + vec2(8.0, 0.0) * s, c + vec2(-7.0, 8.0) * s], Color::new(1.0, 1.0, 1.0, a)); } }
        self.ui.text("ENTER avançar · ESC pular", 1240.0, 702.0, 14.0, Color::new(1.0, 1.0, 1.0, 0.45), Align::Right, false);

        let adv = self.c.pressed(nav::OK) || is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        if self.c.pressed(nav::BACK | nav::START) { self.cutscene_done(kind); return; }
        if adv {
            if !done { t = 99.0; } else { page += 1; t = 0.0; self.audio.sfx("select"); }
        }
        if let Screen::Cutscene { .. } = self.screen { self.screen = Screen::Cutscene { kind, page, t }; }
    }

    fn cutscene_done(&mut self, kind: u8) {
        self.screen = match kind {
            0 => Screen::Stages { cur: 1 },
            1 => Screen::Vs { t: 0.0, step: 0 },
            _ => Screen::Results { winner: 0, cur: 0 },
        };
    }
}
