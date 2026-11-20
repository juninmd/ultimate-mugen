use crate::i18n::tr3;

const RECORD_FILE: &str = "tower-daily.cfg";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DailyRecord {
    pub date_key: String,
    pub cleared: bool,
    pub best_score: i64,
    pub best_floor: usize,
}

fn unix_seconds() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0).max(0.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

pub fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year, m as u32, d as u32)
}

pub fn utc_date_key() -> String {
    let days = (unix_seconds() / 86_400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn seconds_until_reset() -> u64 {
    86_400 - (unix_seconds() % 86_400)
}

pub fn countdown_label() -> String {
    let rem = seconds_until_reset();
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn daily_seed(date_key: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in date_key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub fn tower_theme(seed: u64) -> (&'static str, [f32; 3]) {
    let themes: [(&str, &str, &str, [f32; 3]); 10] = [
        ("TORRE CARMESIM", "CRIMSON SPIRE", "TORRE CARMESÍ", [1.0, 0.28, 0.42]),
        ("MANOPLA DE FERRO", "IRON GAUNTLET", "GUANTELETE DE HIERRO", [1.0, 0.72, 0.2]),
        ("ASCENSÃO SOMBRIA", "SHADOW ASCENT", "ASCENSO SOMBRÍO", [0.75, 0.42, 1.0]),
        ("CIDADELA DO TROVÃO", "THUNDER CITADEL", "CIUDADELA DEL TRUENO", [0.35, 0.88, 1.0]),
        ("TORRE DA LUA DE SANGUE", "BLOOD MOON TOWER", "TORRE LUNA DE SANGRE", [0.95, 0.22, 0.35]),
        ("ESPIRAL CIBER", "CYBER SPIRE", "ESPIRAL CIBER", [0.25, 0.95, 0.72]),
        ("PICO OBSIDIANA", "OBSIDIAN PEAK", "PICO OBSIDIANA", [0.95, 0.52, 0.2]),
        ("PROVAÇÃO DO DRAGÃO", "DRAGON'S TRIAL", "PRUEBA DEL DRAGÓN", [1.0, 0.42, 0.22]),
        ("BASTIÃO FANTASMA", "PHANTOM BASTION", "BASTIÓN FANTASMA", [0.55, 0.72, 1.0]),
        ("CUME DO ÁPICE", "APEX SUMMIT", "CIMA DEL ÁPICE", [1.0, 0.82, 0.3]),
    ];
    let (pt, en, es, accent) = themes[(seed as usize) % themes.len()];
    (tr3(pt, en, es), accent)
}

pub fn load_daily_record(date_key: &str) -> DailyRecord {
    let Some(raw) = crate::storage::read(RECORD_FILE) else {
        return DailyRecord { date_key: date_key.into(), ..DailyRecord::default() };
    };
    let mut rec = DailyRecord::default();
    for line in raw.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        match k.trim() {
            "date" => rec.date_key = v.trim().into(),
            "cleared" => rec.cleared = v.trim() == "1",
            "best_score" => rec.best_score = v.trim().parse().unwrap_or(0),
            "best_floor" => rec.best_floor = v.trim().parse().unwrap_or(0),
            _ => {}
        }
    }
    if rec.date_key != date_key {
        return DailyRecord { date_key: date_key.into(), ..DailyRecord::default() };
    }
    rec
}

pub fn save_daily_record(date_key: &str, cleared: bool, score: i64, floor: usize) -> DailyRecord {
    let prev = load_daily_record(date_key);
    let next = DailyRecord {
        date_key: date_key.into(),
        cleared: prev.cleared || cleared,
        best_score: prev.best_score.max(score),
        best_floor: prev.best_floor.max(floor),
    };
    let text = format!(
        "date={}\ncleared={}\nbest_score={}\nbest_floor={}\n",
        next.date_key, next.cleared as u8, next.best_score, next.best_floor
    );
    crate::storage::write(RECORD_FILE, &text);
    next
}
