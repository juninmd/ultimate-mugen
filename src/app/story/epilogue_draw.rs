use crate::app::App;
use crate::gfx::*;
use crate::i18n::tr3;
use crate::poly;
use crate::story::epilogue::{Beat, Epilogue, Speaker};
use crate::ui::*;

const BAR: f32 = 58.0;
const FLOOR: f32 = 556.0;
const FADE: f32 = 0.6;
const ZOOM: f32 = 1.1;
const NIGHT: Color = Color::new(0.5, 0.56, 0.86, 1.0);
const DAWN: Color = Color::new(1.0, 0.84, 0.7, 1.0);
const DIM: Color = Color::new(0.42, 0.4, 0.48, 1.0);

impl App {
    pub(super) fn draw_story_epilogue(
        &mut self,
        script: &Epilogue,
        beats: &[Beat],
        page: usize,
        t: f32,
        boss: Option<usize>,
    ) -> bool {
        let beat = beats.get(page);
        let home = beat.is_none_or(|beat| beat.home);
        let first_home = beats
            .iter()
            .position(|beat| beat.home)
            .unwrap_or(beats.len());
        let closing = page + 1 >= beats.len();
        let hero = self.sel.0;
        let accent = rgb(self.reg.chars[hero].color);
        let rooftop = crate::story::STAGES[crate::story::STAGES.len() - 1].arena;
        let scene = if home { script.scene.as_str() } else { rooftop };
        let tint = match (home, page == first_home) {
            (false, _) => NIGHT,
            (true, true) => DAWN,
            _ => WHITE,
        };
        self.ui.rect(0.0, 0.0, VW, 720.0, BLACK);
        crate::stages::scenery::want(rooftop);
        crate::stages::scenery::want(&script.scene);
        self.epilogue_backdrop(scene, tint);
        if home {
            let rise = if page == first_home { 0.0 } else { 70.0 };
            let sun = vec2(self.ui.x(1010.0), self.ui.y(300.0 - rise));
            for ring in 0..7 {
                poly::circle(
                    sun,
                    (60.0 + ring as f32 * 34.0) * self.ui.s,
                    Color::new(1.0, 0.82, 0.5, 0.05),
                );
            }
            self.epilogue_wash(
                420.0,
                300.0,
                Color::new(1.0, 0.7, 0.4, 0.0),
                Color::new(1.0, 0.7, 0.4, 0.22),
            );
        } else {
            self.epilogue_wash(
                0.0,
                420.0,
                Color::new(0.02, 0.01, 0.08, 0.7),
                Color::new(0.02, 0.01, 0.08, 0.0),
            );
        }
        self.ui.motes(if home { 30 } else { 45 }, 0.9);
        self.epilogue_cast(beat, home, closing, t, boss);
        if beat.is_none() {
            self.ui
                .rect(0.0, 0.0, VW, 720.0, Color::new(0.0, 0.0, 0.0, 0.45));
        }
        if page == 0 || page == first_home {
            self.ui.rect(
                0.0,
                0.0,
                VW,
                720.0,
                Color::new(0.0, 0.0, 0.0, 1.0 - (t / FADE).min(1.0)),
            );
        }
        self.ui.rect(0.0, 0.0, VW, BAR, BLACK);
        self.ui.rect(0.0, 720.0 - BAR, VW, BAR, BLACK);
        let title = script.title.pick(crate::profile::lang()).to_uppercase();
        self.ui.text(
            tr3("EPÍLOGO", "EPILOGUE", "EPÍLOGO"),
            94.0,
            24.0,
            14.0,
            alpha(accent, 0.95),
            Align::Left,
            true,
        );
        self.ui
            .text(&title, 94.0, 46.0, 20.0, WHITE, Align::Left, true);
        self.ui.text(
            &format!("{}/{}", page + 1, beats.len() + 1),
            1186.0,
            38.0,
            15.0,
            hex("#a49cc0"),
            Align::Right,
            true,
        );
        let hint = tr3(
            "OK/A avançar · ESC/START pular",
            "OK/A next · ESC/START skip",
            "OK/A avanzar · ESC/START saltar",
        );
        self.ui.text(
            hint,
            1240.0,
            700.0,
            14.0,
            Color::new(1.0, 1.0, 1.0, 0.5),
            Align::Right,
            false,
        );
        match beat {
            Some(beat) => self.epilogue_line(beat, t, accent, boss),
            None => {
                self.epilogue_end_card(&title, accent);
                true
            }
        }
    }

    fn epilogue_wash(&self, y: f32, h: f32, top: Color, bottom: Color) {
        let corners = [(0.0, y), (VW, y), (VW, y + h), (0.0, y + h)];
        let quad: Vec<Vec2> = corners
            .iter()
            .map(|&(a, b)| vec2(self.ui.x(a), self.ui.y(b)))
            .collect();
        poly::grad_v(&quad, top, bottom);
    }

    fn epilogue_backdrop(&mut self, scene: &str, tint: Color) {
        let Some(image) = crate::stages::scenery::still(scene)
            .or_else(|| crate::stages::illustration_preview(scene))
        else {
            return;
        };
        let (w, h) = (image.w as f32, image.h as f32);
        let crop_w = w / ZOOM;
        let crop_h = (crop_w * 720.0 / VW).min(h);
        let drift = 0.5 + 0.5 * (self.ui.t * 0.11).sin();
        let texture = self.g.texture(&image, true);
        draw_texture_ex(
            &texture,
            self.ui.x(0.0),
            self.ui.y(0.0),
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(VW * self.ui.s, 720.0 * self.ui.s)),
                source: Some(Rect::new(
                    (w - crop_w) * drift,
                    (h - crop_h) * 0.7,
                    crop_w,
                    crop_h,
                )),
                ..Default::default()
            },
        );
    }

    fn epilogue_cast(
        &mut self,
        beat: Option<&Beat>,
        home: bool,
        closing: bool,
        t: f32,
        boss: Option<usize>,
    ) {
        let hero = self.sel.0;
        let time = self.preview_t;
        let slide = (1.0 - (t / 0.5).min(1.0)).powi(2) * 160.0;
        let speaker = beat.map(|beat| beat.speaker);
        let hero_def = self.reg.chars[hero]
            .peek(1)
            .or_else(|| self.reg.chars[hero].load_preview(1).ok());
        if home {
            let anim = if closing { 181 } else { 0 };
            self.char_tint = WHITE;
            if let Some(def) = &hero_def {
                self.draw_char(def, anim, time, 380.0 - slide, FLOOR, 400.0, false);
            }
            return;
        }
        self.char_tint = if speaker == Some(Speaker::Boss) {
            DIM
        } else {
            WHITE
        };
        if let Some(def) = &hero_def {
            let anim = if speaker == Some(Speaker::Hero) {
                181
            } else {
                0
            };
            self.draw_char(def, anim, time, 360.0 - slide, FLOOR, 400.0, false);
        }
        self.char_tint = if speaker == Some(Speaker::Hero) {
            DIM
        } else {
            WHITE
        };
        let boss_def = boss
            .filter(|&index| !self.reg.chars[index].mugen)
            .and_then(|index| {
                self.reg.chars[index]
                    .peek(1)
                    .or_else(|| self.reg.chars[index].load(1).ok())
            });
        if let Some(def) = &boss_def {
            self.draw_char(def, 5150, time, 930.0 + slide, FLOOR, 420.0, true);
        }
        self.char_tint = WHITE;
    }
}
