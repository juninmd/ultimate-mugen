use super::window::parse_window;
use super::GRID_PRESETS;
use super::*;

pub(super) fn pick<T: std::str::FromStr + PartialOrd + Copy>(v: &str, default: T, max: T) -> T {
    match v.parse::<T>() {
        Ok(n) if n <= max => n,
        Ok(_) => max,
        Err(_) => default,
    }
}

impl Settings {
    pub fn from_text(t: &str) -> Self {
        let mut s = Settings::default();
        for l in t.lines() {
            let Some((k, v)) = l.split_once('=') else {
                continue;
            };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "fullscreen" => s.fullscreen = v == "1",
                "res" => s.res = pick(v, 1, RESOLUTIONS.len() - 1),
                "quality" => s.quality = pick(v, 0, QUALITY.len() - 1),
                "fps" => s.fps = v == "1",
                "fx" => s.fx = pick(v, 2, 2),
                "impact_lv" => s.impact = pick(v, 2, IMPACT.len() - 1),
                "keep_aspect" => s.keep_aspect = v != "0",
                "filter" => s.linear = v == "linear",
                "window" => s.window = parse_window(v),
                "vsync" => s.vsync = v != "0",
                "fps_cap" => s.fps_cap = pick(v, 0, FPS_CAPS.len() - 1),
                "master" => s.master = pick(v, 8, VOLUME_STEPS),
                "music" => s.music = pick(v, 10, VOLUME_STEPS),
                "sfx" => s.sfx = pick(v, 10, VOLUME_STEPS),
                "mute" => s.mute = v == "1",
                "difficulty" => s.difficulty = pick(v, 4, 8).max(1),
                "round_time" => s.round_time = pick(v, 0, ROUND_SECONDS.len() - 1),
                "rounds" => s.rounds = pick(v, 1, ROUNDS_TO_WIN.len() - 1),
                "life" => s.life = pick(v, 2, LIFE_PERCENT.len() - 1),
                "single_round" => s.single_round = v != "0",
                "classic_pushbox" => s.classic_pushbox = v != "0",
                "team_lifeshare" => s.team_life_share = v != "0",
                "gamespeed" => s.game_speed = pick(v, 2, GAME_SPEEDS.len() - 1),
                "survival_recovery" => {
                    s.survival_recovery = pick(v, 2, SURVIVAL_RECOVERY.len() - 1)
                }
                "survival_best" => s.survival_best = v.parse().unwrap_or(0),
                "pal1" => s.pal1 = pick(v, 1, PALETTE_COUNT),
                "pal2" => s.pal2 = pick(v, 1, PALETTE_COUNT),
                "grid" => s.grid = pick(v, 1, GRID_PRESETS.len() - 1),
                "lang" => s.lang = crate::i18n::Lang::from_code(v),

                _ => {}
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        format!(
            "version={}\nfullscreen={}\nres={}\nquality={}\nfps={}\nfx={}\nimpact_lv={}\nkeep_aspect={}\nfilter={}\nwindow={}x{}\nvsync={}\nfps_cap={}\nmaster={}\nmusic={}\nsfx={}\nmute={}\ndifficulty={}\nround_time={}\nrounds={}\nlife={}\nsingle_round={}\nclassic_pushbox={}\nteam_lifeshare={}\ngamespeed={}\nsurvival_recovery={}\nsurvival_best={}\npal1={}\npal2={}\nlang={}\ngrid={}\n",
            FORMAT_VERSION, self.fullscreen as i32, self.res, self.quality, self.fps as i32, self.fx, self.impact, self.keep_aspect as i32, if self.linear { "linear" } else { "nearest" }, self.window.0, self.window.1, self.vsync as i32, self.fps_cap, self.master, self.music, self.sfx, self.mute as i32,
            self.difficulty, self.round_time, self.rounds, self.life, self.single_round as i32, self.classic_pushbox as i32, self.team_life_share as i32, self.game_speed, self.survival_recovery, self.survival_best, self.pal1, self.pal2, self.lang.code(), self.grid,
        )
    }

    pub fn load() -> Self {
        crate::storage::read(FILE)
            .map(|t| Self::from_text(&t))
            .unwrap_or_else(|| Settings { lang: crate::i18n::system::language(), ..Settings::default() })
    }
    pub fn save(&self) {
        crate::storage::write(FILE, &self.to_text());
    }
}
