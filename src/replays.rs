//! On-disk replays: a recorded fight (mode, fighters, stage, rules and the per-tick input stream) saved as a
//! small JSON file. The simulation is deterministic, so replaying the inputs rebuilds the exact same fight.
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "ultimate-mugen-replay";
pub const VERSION: u32 = 1;
pub const SUPERSEDED: &str =
    "Replay incompatível: este arquivo foi gravado por outra versão do jogo.";

#[derive(Clone, Debug, PartialEq)]
pub struct NetplayConfig {
    pub side: i32,
    pub peer: String,
    pub port: u16,
    pub delay: usize,
    pub rollback_window: i32,
    pub players: u8,
    pub spectate: bool,
}

impl NetplayConfig {
    pub fn to_json(&self) -> serde_json::Value {
        json!({ "side": self.side, "peer": self.peer, "port": self.port, "delay": self.delay, "rollback_window": self.rollback_window, "players": self.players, "spectate": self.spectate })
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let o = v.as_object()?;
        Some(NetplayConfig {
            side: o.get("side").and_then(Value::as_i64).unwrap_or(0) as i32,
            peer: o
                .get("peer")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            port: o.get("port").and_then(Value::as_u64).unwrap_or(0) as u16,
            delay: o.get("delay").and_then(Value::as_u64).unwrap_or(0) as usize,
            rollback_window: o
                .get("rollback_window")
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32,
            players: o.get("players").and_then(Value::as_u64).unwrap_or(2) as u8,
            spectate: o.get("spectate").and_then(Value::as_bool).unwrap_or(false),
        })
    }
}

/// One recorded fight, exactly as stored on disk.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredReplay {
    pub label: String,
    pub saved_at: u64,
    pub mode: String,
    pub chars: [String; 2],
    pub stage: String,
    pub palettes: [i32; 2],
    /// Per-member palettes of side 0 in team modes (empty for single fights)
    pub member_palettes: Vec<i32>,
    pub seed: u32,
    /// life ratios the fight started with (Survival carries life between fights)
    pub life: [f32; 2],
    pub teams: Option<[Vec<String>; 2]>,
    pub simul: bool,
    pub tag: bool,
    pub rules: StoredRules,
    /// Netplay session configuration when the fight was played online
    pub netplay: Option<NetplayConfig>,
    /// run-length encoded input stream: (repeat count, one input per root slot)
    pub runs: Vec<(u32, Vec<u32>)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StoredRules {
    pub rounds_to_win: i32,
    pub round_ticks: i32,
    pub life_percent: u32,
    pub single_round: bool,
    pub classic_pushbox: bool,
    pub team_life_share: bool,
    pub game_speed: f32,
}

impl StoredReplay {
    pub fn frames(&self) -> u32 {
        self.runs.iter().map(|r| r.0).sum()
    }

    pub fn seconds(&self) -> f32 {
        self.frames() as f32 / 60.0
    }

    /// Recorded input stream (one row per tick, one column per root slot).
    pub fn stream(&self) -> Vec<Vec<u32>> {
        let mut out = Vec::with_capacity(self.frames() as usize);
        for (n, bits) in &self.runs {
            for _ in 0..*n {
                out.push(bits.clone());
            }
        }
        out
    }

    pub fn from_stream(rec: &[Vec<u32>]) -> Vec<(u32, Vec<u32>)> {
        let mut runs: Vec<(u32, Vec<u32>)> = vec![];
        for f in rec {
            match runs.last_mut() {
                Some((n, bits)) if *bits == *f => *n += 1,
                _ => runs.push((1, f.clone())),
            }
        }
        runs
    }

    pub fn to_json(&self) -> String {
        let slots = self.runs.first().map(|r| r.1.len()).unwrap_or(2);
        let mut flat: Vec<Value> = vec![json!(slots)];
        for (n, bits) in &self.runs {
            flat.push(json!(n));
            for b in bits {
                flat.push(json!(b));
            }
        }
        let teams = self.teams.as_ref().map(|t| json!([t[0], t[1]]));
        json!({
            "format": FORMAT,
            "version": VERSION,
            "label": self.label,
            "saved_at": self.saved_at,
            "mode": self.mode,
            "chars": self.chars,
            "stage": self.stage,
            "palettes": self.palettes,
            "member_palettes": self.member_palettes,
            "seed": self.seed,
            "life": self.life,
            "netplay": self.netplay.as_ref().map(NetplayConfig::to_json),
            "teams": teams,
            "simul": self.simul,
            "tag": self.tag,
            "rules": {
                "rounds_to_win": self.rules.rounds_to_win,
                "round_ticks": self.rules.round_ticks,
                "life_percent": self.rules.life_percent,
                "single_round": self.rules.single_round,
                "classic_pushbox": self.rules.classic_pushbox,
                "team_life_share": self.rules.team_life_share,
                "game_speed": self.rules.game_speed,
            },
            "inputs": flat,
        })
        .to_string()
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("Replay inválido: {e}"))?;
        let o = v
            .as_object()
            .ok_or("Replay inválido: raiz não é um objeto")?;
        if o.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err("Replay inválido: formato desconhecido".into());
        }
        let version = o.get("version").and_then(Value::as_u64).unwrap_or(0);
        if version != VERSION as u64 {
            return Err(format!(
                "{SUPERSEDED} (versão {version}, esperada {VERSION})"
            ));
        }
        let flat = o
            .get("inputs")
            .and_then(Value::as_array)
            .ok_or("Replay inválido: sem quadro de entradas")?;
        let slots = flat.first().and_then(Value::as_u64).unwrap_or(0) as usize;
        if slots == 0 || slots > 64 {
            return Err("Replay inválido: número de lutadores inválido".into());
        }
        let mut runs = vec![];
        let mut i = 1;
        while i < flat.len() {
            let n = flat[i].as_u64().unwrap_or(0) as u32;
            i += 1;
            let Some(bits) = flat.get(i..i + slots) else {
                return Err("Replay inválido: trilha de entradas truncada".into());
            };
            if n == 0 {
                return Err("Replay inválido: trecho vazio".into());
            }
            runs.push((
                n,
                bits.iter()
                    .map(|b| b.as_u64().unwrap_or(0) as u32)
                    .collect(),
            ));
            i += slots;
        }
        if runs.is_empty() {
            return Err("Replay inválido: nenhuma entrada gravada".into());
        }
        let chars = pair_strings(o.get("chars"))?;
        let palettes = pair_ints(o.get("palettes")).unwrap_or([1, 1]);
        let member_palettes = o
            .get("member_palettes")
            .and_then(Value::as_array)
            .map(|v| v.iter().map(|x| x.as_i64().unwrap_or(1) as i32).collect())
            .unwrap_or_default();
        let life = pair_floats(o.get("life")).unwrap_or([1.0, 1.0]);
        let teams = match o.get("teams") {
            None | Some(Value::Null) => None,
            Some(Value::Array(sides)) => {
                let mut out = [vec![], vec![]];
                for (i, side) in sides.iter().take(2).enumerate() {
                    let list = side
                        .as_array()
                        .ok_or("Replay inválido: equipe malformada")?;
                    out[i] = list
                        .iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect();
                }
                Some(out)
            }
            _ => return Err("Replay inválido: equipe malformada".into()),
        };
        let r = o.get("rules").and_then(Value::as_object);
        let rules = StoredRules {
            rounds_to_win: field_i32(r, "rounds_to_win", 2),
            round_ticks: field_i32(r, "round_ticks", 99 * 60),
            life_percent: field_i32(r, "life_percent", 100).max(0) as u32,
            single_round: r
                .and_then(|r| r.get("single_round"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            classic_pushbox: r
                .and_then(|r| r.get("classic_pushbox"))
                .and_then(Value::as_bool)
                .unwrap_or(true),
            team_life_share: r
                .and_then(|r| r.get("team_life_share"))
                .and_then(Value::as_bool)
                .unwrap_or(true),
            game_speed: r
                .and_then(|r| r.get("game_speed"))
                .and_then(Value::as_f64)
                .map(|f| f as f32)
                .unwrap_or(1.0),
        };
        Ok(StoredReplay {
            label: o
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("Replay")
                .to_string(),
            saved_at: o.get("saved_at").and_then(Value::as_u64).unwrap_or(0),
            mode: o
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("Cpu")
                .to_string(),
            chars,
            stage: o
                .get("stage")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            palettes,
            member_palettes,
            seed: o.get("seed").and_then(Value::as_u64).unwrap_or(0) as u32,
            life,
            teams,
            simul: o.get("simul").and_then(Value::as_bool).unwrap_or(false),
            tag: o.get("tag").and_then(Value::as_bool).unwrap_or(false),
            rules,
            netplay: o.get("netplay").and_then(NetplayConfig::from_json),
            runs,
        })
    }
}

fn pair_strings(v: Option<&Value>) -> Result<[String; 2], String> {
    let arr = v
        .and_then(Value::as_array)
        .ok_or("Replay inválido: lutadores ausentes")?;
    match arr.as_slice() {
        [a, b] => Ok([
            a.as_str()
                .ok_or("Replay inválido: lutador malformado")?
                .to_string(),
            b.as_str()
                .ok_or("Replay inválido: lutador malformado")?
                .to_string(),
        ]),
        _ => Err("Replay inválido: lutadores ausentes".into()),
    }
}

fn pair_ints(v: Option<&Value>) -> Option<[i32; 2]> {
    let arr = v.and_then(Value::as_array)?;
    match arr.as_slice() {
        [a, b] => Some([a.as_i64()? as i32, b.as_i64()? as i32]),
        _ => None,
    }
}

fn pair_floats(v: Option<&Value>) -> Option<[f32; 2]> {
    let arr = v.and_then(Value::as_array)?;
    match arr.as_slice() {
        [a, b] => Some([a.as_f64()? as f32, b.as_f64()? as f32]),
        _ => None,
    }
}

fn field_i32(o: Option<&Map<String, Value>>, key: &str, default: i32) -> i32 {
    o.and_then(|o| o.get(key))
        .and_then(Value::as_i64)
        .map(|v| v as i32)
        .unwrap_or(default)
}

/// `replays/` beside the executable (native). Falls back to the working directory when the exe path is unknown.
pub fn dir() -> PathBuf {
    #[cfg(target_os = "android")]
    if let Some(d) = ume_shell::ANDROID_APP
        .get()
        .and_then(|a| a.internal_data_path())
    {
        return d.join("replays");
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("replays")))
        .unwrap_or_else(|| PathBuf::from("replays"))
}

pub fn file_name(replay: &StoredReplay) -> String {
    let slug: String = replay
        .label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    let slug = if slug.is_empty() {
        "replay".to_string()
    } else {
        slug.chars().take(48).collect()
    };
    format!("{slug}-{}.umr.json", replay.saved_at)
}

pub fn save(dir: &Path, replay: &StoredReplay) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Não foi possível criar {}: {e}", dir.display()))?;
    let path = dir.join(file_name(replay));
    std::fs::write(&path, replay.to_json())
        .map_err(|e| format!("Não foi possível gravar o replay: {e}"))?;
    Ok(path)
}

pub fn load(path: &Path) -> Result<StoredReplay, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Não foi possível abrir o replay: {e}"))?;
    let mut r = StoredReplay::from_json(&text)?;
    if r.label.trim().is_empty() {
        r.label = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Replay".into());
    }
    Ok(r)
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub replay: StoredReplay,
}

/// Every readable replay in `dir`, newest first.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut out: Vec<Entry> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|path| load(&path).ok().map(|replay| Entry { path, replay }))
        .collect();
    out.sort_by(|a, b| {
        b.replay
            .saved_at
            .cmp(&a.replay.saved_at)
            .then_with(|| a.replay.label.cmp(&b.replay.label))
    });
    out
}

/// Replays that could not be read, with the reason (shown in the browser so an incompatible file is not silent).
pub fn problems(dir: &Path) -> Vec<(PathBuf, String)> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return vec![];
    };
    read.filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("desync-")))
        .filter_map(|path| match load(&path) { Ok(_) => None, Err(e) => Some((path, e)) })
        .collect()
}

pub fn rename(path: &Path, label: &str) -> Result<PathBuf, String> {
    let mut r = load(path)?;
    r.label = label.trim().to_string();
    if r.label.is_empty() {
        r.label = "Replay".into();
    }
    let out = path.with_file_name(file_name(&r));
    std::fs::write(&out, r.to_json()).map_err(|e| format!("Não foi possível renomear: {e}"))?;
    if out != path {
        let _ = std::fs::remove_file(path);
    }
    Ok(out)
}

pub fn delete(path: &Path) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| format!("Não foi possível apagar: {e}"))
}

/// Keep the newest `keep` files and delete the rest (the recorder never fills the disk).
pub fn prune(dir: &Path, keep: usize) {
    for e in list(dir).into_iter().skip(keep) {
        let _ = std::fs::remove_file(&e.path);
    }
}

/// Write a desync report (netplay config + checksum + recent inputs) so an online desync can be reproduced.
pub fn save_desync(
    dir: &Path,
    config: Option<&NetplayConfig>,
    frame: u32,
    checksum: u64,
    ticks: usize,
    rec: &[Vec<u32>],
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Não foi possível criar {}: {e}", dir.display()))?;
    let path = dir.join(format!("desync-{}.json", now()));
    let tail: Vec<Vec<u32>> = rec.iter().rev().take(120).rev().cloned().collect();
    let body = json!({
        "kind": "ultimate-mugen-desync",
        "version": VERSION,
        "saved_at": now(),
        "config": config.map(NetplayConfig::to_json),
        "frame": frame,
        "checksum": checksum,
        "ticks": ticks,
        "inputs": tail,
    });
    std::fs::write(&path, body.to_string())
        .map_err(|e| format!("Não foi possível gravar o relatório: {e}"))?;
    Ok(path)
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `YYYY-MM-DD HH:MM` in UTC, for the browser (no chrono dependency).
pub fn date_label(epoch: u64) -> String {
    let (mut days, rem) = (epoch / 86_400, epoch % 86_400);
    let (mut y, mut m) = (1970i64, 1u32);
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let len = if m == 2 {
            if leap {
                29
            } else {
                28
            }
        } else if [4, 6, 9, 11].contains(&m) {
            30
        } else {
            31
        };
        if days < len {
            break;
        }
        days -= len;
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    format!(
        "{y:04}-{m:02}-{:02} {:02}:{:02}",
        days + 1,
        rem / 3600,
        (rem % 3600) / 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> StoredReplay {
        let mut rec = Vec::new();
        for i in 0..1200u32 {
            rec.push(vec![
                if i % 7 == 0 { 4 } else { 0 },
                if i % 11 == 0 { 8 } else { 0 },
            ]);
        }
        StoredReplay {
            label: "Kung Fu Man vs Kung Fu Man".into(),
            saved_at: 1_700_000_000,
            mode: "Cpu".into(),
            chars: ["umf:kfm".into(), "umf:kfm".into()],
            stage: "dojo".into(),
            palettes: [1, 2],
            member_palettes: vec![3, 4],
            seed: 0x2545_f491,
            life: [0.55, 1.0],
            teams: None,
            simul: false,
            tag: false,
            rules: StoredRules {
                rounds_to_win: 2,
                round_ticks: 5940,
                life_percent: 100,
                single_round: false,
                classic_pushbox: true,
                team_life_share: true,
                game_speed: 1.0,
            },
            netplay: Some(NetplayConfig {
                side: 0,
                peer: "127.0.0.1:7001".into(),
                port: 7000,
                delay: 2,
                rollback_window: 12,
                players: 2,
                spectate: false,
            }),
            runs: StoredReplay::from_stream(&rec),
        }
    }

    #[test]
    fn json_round_trip_keeps_every_field() {
        let r = sample();
        let back = StoredReplay::from_json(&r.to_json()).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.frames(), 1200);
    }

    #[test]
    fn a_stored_replay_rebuilds_the_same_input_stream() {
        let r = sample();
        let back = StoredReplay::from_json(&r.to_json()).unwrap();
        assert_eq!(
            back.stream(),
            r.stream(),
            "reloading preserves the recorded inputs"
        );
        assert_eq!(back.stream().len(), 1200);
    }

    #[test]
    fn run_length_encoding_keeps_held_buttons_small() {
        let rec: Vec<Vec<u32>> = (0..600).map(|_| vec![4, 0]).collect();
        let runs = StoredReplay::from_stream(&rec);
        assert_eq!(runs.len(), 1, "600 identical frames collapse into one run");
        assert_eq!(runs[0], (600, vec![4, 0]));
    }

    #[test]
    fn team_replays_keep_every_slot() {
        let mut r = sample();
        r.tag = true;
        r.teams = Some([
            vec!["umf:kfm".into(), "umf:kfm".into()],
            vec!["umf:kfm".into()],
        ]);
        r.runs = StoredReplay::from_stream(&vec![vec![1, 2, 3, 4]; 90]);
        let back = StoredReplay::from_json(&r.to_json()).unwrap();
        assert_eq!(back.stream().len(), 90);
        assert!(
            back.stream().iter().all(|f| f.len() == 4),
            "all four slots survive the round trip"
        );
        assert_eq!(back.teams.as_ref().unwrap()[0].len(), 2);
    }

    #[test]
    fn another_version_is_rejected_with_a_clear_message() {
        let text = sample()
            .to_json()
            .replace("\"version\":1", "\"version\":99");
        let err = StoredReplay::from_json(&text).unwrap_err();
        assert!(err.contains("versão 99"), "unexpected error: {err}");
        assert!(err.contains(SUPERSEDED));
    }

    #[test]
    fn a_malformed_file_is_rejected() {
        assert!(StoredReplay::from_json("not json").is_err());
        assert!(StoredReplay::from_json("{\"format\":\"other\"}").is_err());
        let mut v: Value = serde_json::from_str(&sample().to_json()).unwrap();
        v["inputs"].as_array_mut().unwrap().pop();
        assert!(
            StoredReplay::from_json(&v.to_string()).is_err(),
            "a truncated stream is rejected"
        );
    }

    fn scratch(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("ume-replays-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn save_list_rename_and_delete() {
        let dir = scratch("crud");
        let mut a = sample();
        let pa = save(&dir, &a).unwrap();
        a.saved_at += 60;
        a.label = "Segunda luta".into();
        let pb = save(&dir, &a).unwrap();
        assert!(pa.exists() && pb.exists());

        let entries = list(&dir);
        assert_eq!(entries.len(), 2, "both files are listed");
        assert_eq!(entries[0].replay.label, "Segunda luta", "newest first");

        let renamed = rename(&entries[0].path, "Luta renomeada").unwrap();
        assert!(renamed.exists());
        assert!(
            !entries[0].path.exists(),
            "the old file is gone after rename"
        );
        let after = list(&dir);
        assert!(after.iter().any(|e| e.replay.label == "Luta renomeada"));

        delete(&after[0].path).unwrap();
        assert_eq!(list(&dir).len(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_incompatible_file_is_reported_by_name() {
        let dir = scratch("broken");
        std::fs::write(
            dir.join("old-1.umr.json"),
            sample()
                .to_json()
                .replace("\"version\":1", "\"version\":42"),
        )
        .unwrap();
        save(&dir, &sample()).unwrap();
        let problems = problems(&dir);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].1.contains("versão 42"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prune_keeps_the_newest_files() {
        let dir = scratch("prune");
        for i in 0..5u64 {
            let mut r = sample();
            r.saved_at = 1_700_000_000 + i * 60;
            r.label = format!("luta {i}");
            save(&dir, &r).unwrap();
        }
        prune(&dir, 2);
        let entries = list(&dir);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].replay.label, "luta 4");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn netplay_config_survives_the_round_trip() {
        let r = sample();
        let back = StoredReplay::from_json(&r.to_json()).unwrap();
        assert_eq!(back.netplay, r.netplay);
        assert_eq!(back.netplay.as_ref().unwrap().rollback_window, 12);
        assert_eq!(back.netplay.as_ref().unwrap().peer, "127.0.0.1:7001");
    }

    #[test]
    fn a_desync_report_records_the_config_checksum_and_recent_inputs() {
        let dir = scratch("desync");
        let cfg = NetplayConfig {
            side: 1,
            peer: "10.0.0.2:7000".into(),
            port: 7001,
            delay: 3,
            rollback_window: 12,
            players: 2,
            spectate: false,
        };
        let rec: Vec<Vec<u32>> = (0..300).map(|i| vec![i % 8, i % 5]).collect();
        let path = save_desync(&dir, Some(&cfg), 900, 0xdead_beef, rec.len(), &rec).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["frame"], 900);
        assert_eq!(v["checksum"], 0xdead_beef_u64);
        assert_eq!(v["config"]["peer"], "10.0.0.2:7000");
        assert_eq!(v["config"]["delay"], 3);
        assert_eq!(
            v["inputs"].as_array().unwrap().len(),
            120,
            "the report keeps the last 120 frames"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn files_are_named_by_label_and_timestamp() {
        let r = sample();
        assert_eq!(
            file_name(&r),
            "kung-fu-man-vs-kung-fu-man-1700000000.umr.json"
        );
        let mut weird = r.clone();
        weird.label = "  //  ".into();
        assert!(file_name(&weird).starts_with("replay-"));
    }

    #[test]
    fn dates_are_formatted_in_utc() {
        assert_eq!(date_label(0), "1970-01-01 00:00");
        assert_eq!(date_label(1_700_000_000), "2023-11-14 22:13");
    }
}
