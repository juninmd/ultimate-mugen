use super::{combo_scale_for, Match, COMBO_WINDOW};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboTelemetry {
    pub hits: i32,
    pub damage: f32,
    pub next_normal_scale: f32,
    pub next_hyper_scale: f32,
    pub window_left: i32,
    pub window_ratio: f32,
    pub target_id: Option<u32>,
}

impl Default for ComboTelemetry {
    fn default() -> Self {
        ComboTelemetry {
            hits: 0,
            damage: 0.0,
            next_normal_scale: 1.0,
            next_hyper_scale: 1.0,
            window_left: 0,
            window_ratio: 0.0,
            target_id: None,
        }
    }
}

impl Match {
    pub fn combo_telemetry(&self, side: usize) -> ComboTelemetry {
        let Some(f) = self
            .fighters
            .iter()
            .find(|f| !f.is_helper && f.side == side)
        else {
            return ComboTelemetry::default();
        };
        let hits = f.combo_count.max(0);
        let window_left = f.combo_timer.max(0);
        let window_ratio = if COMBO_WINDOW > 0 {
            (window_left as f32 / COMBO_WINDOW as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        ComboTelemetry {
            hits,
            damage: f.combo_damage.max(0.0),
            next_normal_scale: combo_scale_for(hits, false),
            next_hyper_scale: combo_scale_for(hits, true),
            window_left,
            window_ratio,
            target_id: f.combo_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chardef::CharDef;
    use crate::common::{common_def, flat_stage};
    use std::sync::Arc;

    #[test]
    fn combo_telemetry_reports_live_proration_and_window_ratio() {
        let mut m = Match::new(common_def(), None, flat_stage());
        let def = Arc::new(CharDef::empty("p1"));
        m.fighters.push(crate::fighter::Fighter::new(1, def.clone(), 0, 1));
        m.fighters.push(crate::fighter::Fighter::new(2, def, 1, 2));

        let idle = m.combo_telemetry(0);
        assert_eq!(idle.hits, 0);
        assert_eq!(idle.next_normal_scale, 1.0);
        assert_eq!(idle.window_ratio, 0.0);

        m.fighters[0].combo_count = 6;
        m.fighters[0].combo_damage = 312.5;
        m.fighters[0].combo_timer = COMBO_WINDOW / 2;
        m.fighters[0].combo_target = Some(m.fighters[1].id);

        let mid = m.combo_telemetry(0);
        assert_eq!(mid.hits, 6);
        assert!((mid.damage - 312.5).abs() < 0.01);
        assert!((mid.next_normal_scale - 0.5).abs() < 0.01);
        assert!((mid.next_hyper_scale - 0.5).abs() < 0.01);
        assert!((mid.window_ratio - 0.5).abs() < 0.02);
        assert_eq!(mid.target_id, Some(m.fighters[1].id));

        m.fighters[0].combo_count = 10;
        let deep = m.combo_telemetry(0);
        assert!((deep.next_normal_scale - 0.25).abs() < 0.01);
        assert!((deep.next_hyper_scale - 0.5).abs() < 0.01);
    }
}
