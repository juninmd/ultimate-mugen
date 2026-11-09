use crate::art::{background, slab};
use egui::{pos2, vec2, Color32, FontId, Rect};
use egui_phosphor::regular as ic;

pub const ITEMS: [(&str, &str, &str); 8] = [
    (ic::PLAY, "Arcade", "Fight a sequence of opponents"),
    (ic::ROBOT, "Versus CPU", "You against the computer"),
    (ic::USERS, "2 Players", "Shared keyboard or two controllers"),
    (
        ic::TARGET,
        "Training",
        "Training dummy, infinite bar, hitboxes",
    ),
    (
        ic::PACKAGE,
        "Marketplace",
        "Download characters, stages and music",
    ),
    (
        ic::GAME_CONTROLLER,
        "Controls",
        "Keyboard, Xbox, PlayStation, touch",
    ),
    (
        ic::SLIDERS_HORIZONTAL,
        "Settings",
        "Video, audio, gameplay and language",
    ),
    (ic::POWER, "Quit", "Close the game"),
];

const ACCENTS: [Color32; 4] = [
    Color32::from_rgb(0xff, 0xb2, 0x1f),
    Color32::from_rgb(0x22, 0xe8, 0xff),
    Color32::from_rgb(0xff, 0x4d, 0x6d),
    Color32::from_rgb(0x6d, 0xff, 0x9e),
];

pub fn back_out(t: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

pub fn cubic_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub struct Menu {
    pub cur: usize,
    from: f32,
    to: f32,
    started: f64,
    pub settings: bool,
    pub volume: f32,
    pub glass: bool,
    pub quit: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            cur: 1,
            from: 1.0,
            to: 1.0,
            started: 0.0,
            settings: false,
            volume: 0.7,
            glass: true,
            quit: false,
        }
    }

    fn select(&mut self, i: usize, now: f64) {
        if i == self.cur {
            return;
        }
        self.from = self.row_pos(now);
        self.to = i as f32;
        self.cur = i;
        self.started = now;
    }

    fn row_pos(&self, now: f64) -> f32 {
        let t = ((now - self.started) / 0.38).clamp(0.0, 1.0) as f32;
        self.from + (self.to - self.from) * back_out(t)
    }

    fn accent(&self, now: f64) -> Color32 {
        let k = ((now - self.started) / 0.35).clamp(0.0, 1.0) as f32;
        let a = ACCENTS[(self.from.round() as usize) % ACCENTS.len()];
        let b = ACCENTS[self.cur % ACCENTS.len()];
        let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * cubic_out(k)) as u8;
        Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
    }

    pub fn ui(&mut self, root: &mut egui::Ui, now: f64) {
        let ctx = root.ctx().clone();
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowDown) {
                self.select((self.cur + 1) % ITEMS.len(), now);
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.select((self.cur + ITEMS.len() - 1) % ITEMS.len(), now);
            }
        });
        let accent = self.accent(now);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let full = ui.max_rect();
                let s = full.width() / 1280.0;
                let p = ui.painter().clone();
                background(&p, full, accent, now);
                let panel = Rect::from_min_max(full.min, pos2(full.min.x + 600.0 * s, full.max.y));
                p.rect_filled(panel, 0.0, Color32::from_rgba_unmultiplied(10, 8, 26, 150));
                p.rect_filled(
                    Rect::from_min_size(
                        pos2(panel.max.x, full.min.y),
                        vec2(3.0 * s, full.height()),
                    ),
                    0.0,
                    accent,
                );
                self.items(ui, full, s, accent, now);
                self.right(&p, full, s, accent);
            });
        self.settings_window(&ctx);
    }

    fn items(&mut self, ui: &mut egui::Ui, full: Rect, s: f32, accent: Color32, now: f64) {
        let step = 58.0 * s;
        let top = full.min.y + 190.0 * s;
        let h = 50.0 * s;
        let hl = top + self.row_pos(now) * step;
        let p = ui.painter().clone();
        slab(
            &p,
            62.0 * s + full.min.x,
            hl,
            500.0 * s,
            h,
            18.0 * s,
            accent,
        );
        for (i, (icon, label, _)) in ITEMS.iter().enumerate() {
            let y = top + i as f32 * step;
            let rect = Rect::from_min_size(pos2(full.min.x + 40.0 * s, y), vec2(540.0 * s, h));
            let r = ui.interact(rect, ui.id().with(i), egui::Sense::click());
            if r.hovered() && ui.input(|x| x.pointer.delta() != vec2(0.0, 0.0)) {
                self.select(i, now);
            }
            if r.clicked() {
                self.select(i, now);
                self.settings = ITEMS[i].1 == "Settings";
                self.quit = ITEMS[i].1 == "Quit";
            }
            let on = i == self.cur;
            let dx = if on { 22.0 * s } else { 0.0 };
            if !on {
                slab(
                    &p,
                    rect.min.x,
                    y,
                    440.0 * s,
                    h,
                    18.0 * s,
                    Color32::from_white_alpha(18),
                );
            }
            let ink = if on {
                Color32::from_rgb(0x12, 0x05, 0x1f)
            } else {
                Color32::from_white_alpha(210)
            };
            p.text(
                pos2(rect.min.x + dx + 22.0 * s, y + h * 0.5),
                egui::Align2::LEFT_CENTER,
                *icon,
                FontId::proportional(24.0 * s),
                ink,
            );
            p.text(
                pos2(rect.min.x + dx + 62.0 * s, y + h * 0.5),
                egui::Align2::LEFT_CENTER,
                *label,
                FontId::proportional(if on { 26.0 } else { 23.0 } * s),
                ink,
            );
        }
    }

    fn right(&self, p: &egui::Painter, full: Rect, s: f32, accent: Color32) {
        let (_, label, desc) = ITEMS[self.cur];
        let x = full.min.x + 1220.0 * s;
        p.text(
            pos2(x, full.min.y + 590.0 * s),
            egui::Align2::RIGHT_BOTTOM,
            label.to_uppercase(),
            FontId::proportional(64.0 * s),
            Color32::WHITE,
        );
        p.rect_filled(
            Rect::from_min_size(
                pos2(x - 400.0 * s, full.min.y + 606.0 * s),
                vec2(400.0 * s, 4.0 * s),
            ),
            0.0,
            accent,
        );
        p.text(
            pos2(x, full.min.y + 640.0 * s),
            egui::Align2::RIGHT_BOTTOM,
            desc,
            FontId::proportional(22.0 * s),
            Color32::from_white_alpha(230),
        );
        p.text(
            pos2(full.min.x + 40.0 * s, full.max.y - 20.0 * s),
            egui::Align2::LEFT_BOTTOM,
            "egui spike · wgpu 30 · winit 0.30",
            FontId::proportional(16.0 * s),
            Color32::from_rgb(0xa4, 0x9c, 0xc0),
        );
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.settings;
        egui::Window::new(format!("{} Settings", ic::SLIDERS_HORIZONTAL))
            .open(&mut open)
            .default_pos(pos2(700.0, 180.0))
            .show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));
                ui.checkbox(&mut self.glass, "Glass HUD");
            });
        self.settings = open;
    }
}
