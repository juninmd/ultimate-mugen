use super::Vfx;
use crate::story::objects::catalog;
use ume_engine::matchs::Match;
use ume_engine::stage_objects::{ObjectEvent, StageZoneKind};

const HEAL_COLOR: [f32; 3] = [0.5, 1.0, 0.55];
const POWER_COLOR: [f32; 3] = [1.0, 0.86, 0.3];
const SCORE_COLOR: [f32; 3] = [0.55, 0.9, 1.0];
const SMOKE_COLOR: [f32; 3] = [0.32, 0.3, 0.34];
const SMOKE_EVERY_TICKS: i64 = 5;
const SMOKING_DAMAGE: f32 = 0.45;

impl Vfx {
    pub fn stage_objects(&mut self, m: &mut Match) {
        self.story_juice(m);
        if m.objects.is_idle() {
            return;
        }
        let cat = catalog();
        if m.game_time % SMOKE_EVERY_TICKS == 0 {
            for b in m
                .objects
                .breakables
                .iter()
                .filter(|b| b.solid && b.damage_ratio() >= SMOKING_DAMAGE)
            {
                let heat = b.damage_ratio();
                let x = b.x + b.half_width * 0.72;
                let y = b.lane - b.height * 0.75;
                self.dust(
                    x,
                    y,
                    1 + (heat * 2.0) as usize,
                    0.5,
                    1.6 + heat,
                    SMOKE_COLOR,
                );
                if b.broken && m.game_time % (SMOKE_EVERY_TICKS * 3) == 0 {
                    self.spark_burst(x, y, 1.0, 2, 1.6, [1.0, 0.6, 0.2], 3.1);
                }
            }
        }
        for k in 0..m.objects.events.len() {
            match m.objects.events[k] {
                ObjectEvent::Struck {
                    index,
                    x,
                    y,
                    lane,
                    heavy,
                } => {
                    let kind = m
                        .objects
                        .breakables
                        .get(index)
                        .and_then(|b| cat.kind(b.kind));
                    let color = kind.map_or([1.0, 0.9, 0.6], |k| k.debris);
                    let metal = kind.is_some_and(|k| k.sound == "metal");
                    self.spark_burst(
                        x,
                        y + lane,
                        1.0,
                        if heavy { 9 } else { 5 },
                        2.6,
                        [1.0, 0.92, 0.6],
                        3.1,
                    );
                    self.debris(x, y + lane, if heavy { 5 } else { 2 }, color);
                    self.trauma = (self.trauma + if heavy { 0.22 } else { 0.1 }).min(1.0);
                    self.sfx.push(match (metal, heavy) {
                        (true, false) => "guard",
                        (true, true) => "hitheavy",
                        (false, false) => "hit",
                        (false, true) => "hitheavy",
                    });
                }
                ObjectEvent::Broken { index } => {
                    let Some(b) = m.objects.breakables.get(index) else {
                        continue;
                    };
                    let kind = cat.kind(b.kind);
                    let color = kind.map_or([0.8, 0.8, 0.8], |k| k.debris);
                    let (x, feet, mid) = (b.x, b.lane, b.lane - b.height * 0.5);
                    let big = b.solid;
                    let spread = b.half_width;
                    for step in [-0.6f32, 0.0, 0.6] {
                        self.debris(x + spread * step, mid, if big { 12 } else { 6 }, color);
                    }
                    self.dust(
                        x,
                        feet,
                        if big { 22 } else { 10 },
                        2.4,
                        0.6,
                        [0.82, 0.78, 0.72],
                    );
                    self.ring(
                        x,
                        mid,
                        if big { 120.0 } else { 44.0 },
                        0.4,
                        1.6,
                        [1.0, 0.9, 0.7],
                    );
                    self.trauma = (self.trauma + if big { 0.8 } else { 0.3 }).min(1.0);
                    if big {
                        self.flash = self.flash.max(0.5);
                        self.spark_burst(x, mid, 1.0, 26, 4.2, [1.0, 0.7, 0.25], 3.1);
                        self.sfx.push("ko");
                    }
                    self.sfx.push("guardbreak");
                }
                ObjectEvent::Blast { x, lane, radius, .. } => {
                    let mid = lane - 26.0;
                    self.ring(x, mid, radius.max(60.0), 0.45, 2.2, [1.0, 0.72, 0.22]);
                    self.ring(x, mid, radius * 0.65, 0.32, 1.4, [1.0, 0.96, 0.75]);
                    self.glow(x, mid, radius * 0.55, 0.35, [1.0, 0.65, 0.2]);
                    self.spark_burst(x, mid, 1.0, 22, 4.4, [1.0, 0.78, 0.28], 3.1);
                    self.dust(x, lane, 14, 2.6, 0.8, [0.55, 0.48, 0.44]);
                    self.flash = self.flash.max(0.38);
                    self.chroma = self.chroma.max(0.45);
                    self.trauma = (self.trauma + 0.55).min(1.0);
                    self.sfx.push("burst");
                }
                ObjectEvent::Dropped { x, lane, .. } => {
                    self.glow(x, lane - 20.0, 16.0, 0.3, [1.0, 0.95, 0.7]);
                }
                ObjectEvent::Collected { spec, x, lane, .. } => {
                    let y = lane - 34.0;
                    let (text, color, sound) = if spec.heal > 0.0 {
                        (
                            format!("+{}%", (spec.heal * 100.0).round() as i32),
                            HEAL_COLOR,
                            "meter",
                        )
                    } else if spec.power > 0.0 {
                        (
                            crate::i18n::tr("+ENERGIA", "+POWER").to_string(),
                            POWER_COLOR,
                            "meter",
                        )
                    } else {
                        (format!("+{}", spec.score as i64), SCORE_COLOR, "confirm")
                    };
                    self.float(x, y, text, 8.0, color);
                    self.ring(x, lane - 12.0, 30.0, 0.35, 1.4, color);
                    self.spark_burst(x, lane - 12.0, 1.0, 8, 1.8, color, 3.1);
                    self.sfx.push(sound);
                }
                ObjectEvent::GoalCleared { ticks_left } => {
                    let seconds = (ticks_left as f32 / 60.0).ceil() as i32;
                    self.banner(
                        crate::story::text::bonus_cleared(),
                        &crate::story::text::bonus_time_left(seconds),
                        [1.0, 0.85, 0.3],
                    );
                    self.sfx.push("perfect");
                }
                ObjectEvent::GoalTimedOut => {
                    self.banner(crate::story::text::bonus_timeout(), "", [1.0, 0.5, 0.4]);
                    self.sfx.push("back");
                }
                ObjectEvent::ZoneTriggered { index, x, lane, .. } => {
                    let color = m
                        .objects
                        .zones
                        .get(index)
                        .map(|z| match z.kind {
                            StageZoneKind::Updraft { .. } => [0.55, 0.92, 1.0],
                            StageZoneKind::PowerWell { .. } => POWER_COLOR,
                            StageZoneKind::BurnHazard { .. } => [1.0, 0.36, 0.14],
                            StageZoneKind::IceFloor { .. } => [0.7, 0.9, 1.0],
                        })
                        .unwrap_or(SCORE_COLOR);
                    self.ring(x, lane - 14.0, 28.0, 0.28, 1.3, color);
                    self.spark_burst(x, lane - 14.0, 1.0, 6, 1.6, color, 2.6);
                }
            }
        }
        m.objects.events.clear();
    }
}
