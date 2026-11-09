//! Fight loading: the step pipeline's accounting and the sprite atlas warm-up.
//!
//! The warm-up copies sprite sheets into the render atlases in fixed-size chunks, one chunk per
//! loading step (behind the VS screen) or per yielded frame (`App::start_fight`), so no single frame
//! pays for a whole fighter. A 78-fighter M.U.G.E.N port in the browser solved the same problem by
//! letting every sprite stream on first draw; we keep the atlas but pay for it a frame at a time.
use super::*;

/// Bytes of sprite art copied into the atlases per warm-up frame.
///
/// Measured on a desktop GPU: a 1 MiB chunk (individual `write_texture` calls, one per sprite) costs
/// 20-40 ms, so the budget is half that to stay near a 60 Hz frame. A whole native fighter at the
/// 50 % atlas cap (~33 MB of RGBA art) still fits inside the 1.8 s VS splash.
const WARM_BUDGET: u64 = 512 << 10;

/// Sprite atlas warm-up: every distinct fighter art, the sprite keys to copy, and how far it got.
#[derive(Default)]
pub struct Warm {
    jobs: Vec<(Arc<CharDef>, Vec<(i32, i32)>, usize)>,
}

impl App {
    pub(super) fn load_total(&self) -> u8 {
        (self.load_plan().len() + 4) as u8
    }

    pub(super) fn load_label(&self, step: u8) -> String {
        let plan = self.load_plan();
        match plan.get(step as usize) {
            Some((_, _, r)) => format!("{} {}…", crate::i18n::t("common.loading"), self.reg.chars[*r].name),
            None => [
                crate::i18n::tr3("Montando o cenário…", "Building the stage…", "Montando el escenario…"),
                crate::i18n::tr3("Preparando a luta…", "Preparing the fight…", "Preparando el combate…"),
                crate::i18n::tr3("Sintetizando a música…", "Synthesizing the music…", "Sintetizando la música…"),
                crate::i18n::tr3("Enviando sprites…", "Uploading sprites…", "Enviando sprites…"),
            ][(step as usize - plan.len()).min(3)]
            .to_string(),
        }
    }

    /// One chunk of the sprite atlas warm-up; false while there is still art to copy.
    ///
    /// A fight whose roster did not change keeps the atlases it already filled, so a rematch copies
    /// nothing instead of flushing the texture cache and uploading everything again.
    pub(super) fn warm_step(&mut self) -> bool {
        ume_shell::watchdog::label("load:sprites");
        if self.pending.warm.jobs.is_empty() {
            let Some(sess) = self.pending.sess.as_ref() else {
                return true;
            };
            let mut jobs: Vec<(Arc<CharDef>, Vec<(i32, i32)>, usize)> = vec![];
            let mut key = String::new();
            for f in &sess.m.fighters {
                if jobs.iter().any(|(d, _, _)| d.id == f.def.id) {
                    continue;
                }
                key.push_str(&f.def.id);
                key.push(';');
                let mut keys: Vec<(i32, i32)> = f.def.sprites.keys().copied().collect();
                keys.sort_unstable();
                jobs.push((f.def.clone(), keys, 0));
            }
            if key != self.warm_key {
                self.g.trim(0);
                self.warm_key = key;
            }
            self.pending.warm.jobs = jobs;
        }
        let mut all = true;
        for (def, keys, at) in &mut self.pending.warm.jobs {
            if *at >= keys.len() {
                continue;
            }
            *at = self.g.prefetch_step(def, keys, *at, WARM_BUDGET);
            if *at < keys.len() {
                all = false;
            }
        }
        all
    }

    /// Fraction of the warm-up already copied, for a progress bar.
    pub(super) fn warm_progress(&self) -> f32 {
        let (mut done, mut total) = (0u64, 0u64);
        for (_, keys, at) in &self.pending.warm.jobs {
            done += *at as u64;
            total += keys.len() as u64;
        }
        if total == 0 {
            return 1.0;
        }
        (done as f32 / total as f32).clamp(0.0, 1.0)
    }
}
