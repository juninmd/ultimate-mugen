pub mod dialogue;

#[derive(Clone, Debug, PartialEq)]
pub struct BossToast {
    pub title: String,
    pub subtitle: String,
    pub timer: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BossProgression {
    pub clears: usize,
    pub unlocked_oboro: bool,
    pub toast: Option<BossToast>,
}

impl BossProgression {
    pub fn load() -> Self {
        let mut prog = Self::default();
        if let Some(text) = crate::storage::read("boss_save.txt") {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else {
                    continue;
                };
                match k.trim() {
                    "clears" => prog.clears = v.trim().parse().unwrap_or(0),
                    "unlocked_oboro" => prog.unlocked_oboro = v.trim() == "1",
                    _ => {}
                }
            }
        }
        if prog.clears > 0 {
            prog.unlocked_oboro = true;
        }
        prog
    }

    pub fn save(&self) {
        let text = format!(
            "clears={}\nunlocked_oboro={}\n",
            self.clears,
            if self.unlocked_oboro { 1 } else { 0 }
        );
        crate::storage::write("boss_save.txt", &text);
    }

    pub fn is_unlocked(&self, id: &str) -> bool {
        let clean = id.strip_prefix("umf:").unwrap_or(id);
        if clean == "oboro" {
            self.unlocked_oboro || self.clears > 0
        } else {
            true
        }
    }

    pub fn current_boss_id(&self, player_id: &str) -> &'static str {
        let clean = player_id.strip_prefix("umf:").unwrap_or(player_id);
        let boss = match self.clears % 3 {
            0 => "oboro",
            1 => "garan",
            _ => "valdric",
        };
        if boss == clean {
            match boss {
                "oboro" => "garan",
                "garan" => "valdric",
                "valdric" => "garan",
                _ => "oboro",
            }
        } else {
            boss
        }
    }

    pub fn boss_stage(boss_id: &str) -> &'static str {
        match boss_id.strip_prefix("umf:").unwrap_or(boss_id) {
            "garan" => "volcano",
            "valdric" => "neon",
            _ => "spirit",
        }
    }

    pub fn on_clear(&mut self, player_id: &str, defeated_boss: &str) -> Option<BossToast> {
        let clean_boss = defeated_boss.strip_prefix("umf:").unwrap_or(defeated_boss);
        let was_locked = !self.is_unlocked("oboro");
        if clean_boss == "oboro" || was_locked {
            self.unlocked_oboro = true;
        }
        self.clears += 1;
        self.save();
        let next_id = self.current_boss_id(player_id);
        let next_name = match next_id {
            "garan" => "GARAN",
            "valdric" => "VALDRIC",
            _ => "OBORO",
        };
        let toast = if was_locked {
            Some(BossToast {
                title: crate::i18n::t("boss.unlocked_oboro").to_string(),
                subtitle: crate::i18n::tf("boss.next_villain", &[("{boss}", next_name)]),
                timer: 5.0,
            })
        } else {
            Some(BossToast {
                title: crate::i18n::t("boss.cleared").to_string(),
                subtitle: crate::i18n::tf("boss.next_villain", &[("{boss}", next_name)]),
                timer: 4.0,
            })
        };
        self.toast = toast.clone();
        toast
    }

    pub fn tick_toast(&mut self, dt: f32) {
        if let Some(t) = self.toast.as_mut() {
            t.timer -= dt;
            if t.timer <= 0.0 {
                self.toast = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_progression_rotates_and_unlocks_oboro() {
        let mut prog = BossProgression::default();
        assert_eq!(prog.clears, 0);
        assert!(!prog.is_unlocked("oboro"));
        assert_eq!(prog.current_boss_id("kaito"), "oboro");

        let toast1 = prog.on_clear("kaito", "oboro");
        assert!(prog.is_unlocked("oboro"));
        assert_eq!(prog.clears, 1);
        assert_eq!(prog.current_boss_id("kaito"), "garan");
        assert!(toast1.is_some());

        assert_eq!(prog.current_boss_id("garan"), "valdric");

        let toast2 = prog.on_clear("kaito", "garan");
        assert_eq!(prog.clears, 2);
        assert_eq!(prog.current_boss_id("kaito"), "valdric");
        assert!(toast2.is_some());

        assert_eq!(prog.current_boss_id("valdric"), "garan");

        prog.on_clear("kaito", "valdric");
        assert_eq!(prog.clears, 3);
        assert_eq!(prog.current_boss_id("kaito"), "oboro");
    }

    #[test]
    fn boss_stages_match_lore() {
        assert_eq!(BossProgression::boss_stage("oboro"), "spirit");
        assert_eq!(BossProgression::boss_stage("garan"), "volcano");
        assert_eq!(BossProgression::boss_stage("valdric"), "neon");
    }

    #[test]
    fn dialogue_scripts_generate_all_boss_variants() {
        for kind in 0..=2 {
            for boss in ["oboro", "garan", "valdric"] {
                let s = dialogue::script(kind, "Kaito", "Win", boss, boss, "Intro");
                assert!(!s.is_empty());
            }
        }
    }
}
