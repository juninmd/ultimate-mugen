use crate::debug_fight::load_fighter;
use serde_json::{json, Value};
use ume_engine::chardef::{Animation, CharDef, StateDef};

#[derive(Clone, Debug)]
pub struct MoveFrameData {
    pub state: i32,
    pub anim: i32,
    pub state_type: String,
    pub startup: i32,
    pub active: i32,
    pub recovery: i32,
    pub total: i32,
    pub damage: i32,
    pub guard_flag: String,
    pub hit_adv: Option<i32>,
    pub block_adv: Option<i32>,
    pub max_reach_px: f32,
}

#[derive(Clone, Debug)]
pub struct FighterFrameData {
    pub name: String,
    pub moves: Vec<MoveFrameData>,
}

fn parse_first_int(raw: Option<&str>) -> Option<i32> {
    raw?.split(',')
        .next()?
        .trim()
        .parse::<f64>()
        .ok()
        .map(|v| v.round() as i32)
}

fn analyze_anim(anim: &Animation) -> (i32, i32, i32, i32, f32) {
    let mut elapsed = 0;
    let mut first_active: Option<i32> = None;
    let mut active_ticks = 0;
    let mut last_active_end = 0;
    let mut max_reach = 0.0_f32;

    for fr in &anim.frames {
        let dur = if fr.time < 0 { 1 } else { fr.time };
        if !fr.clsn1.is_empty() {
            if first_active.is_none() {
                first_active = Some(elapsed + 1);
            }
            active_ticks += dur;
            last_active_end = elapsed + dur;
            for b in &fr.clsn1 {
                max_reach = max_reach.max(b[2]);
            }
        }
        elapsed += dur;
    }

    let startup = first_active.unwrap_or(0);
    let recovery = if first_active.is_some() {
        (elapsed - last_active_end).max(0)
    } else {
        0
    };
    (startup, active_ticks, recovery, elapsed, max_reach)
}

fn extract_state(def: &CharDef, sd: &StateDef) -> Option<MoveFrameData> {
    let movetype = sd
        .param("movetype")
        .map(|p| p.raw.trim().to_ascii_uppercase())
        .unwrap_or_default();
    if movetype != "A" {
        return None;
    }

    let state_type = sd
        .param("type")
        .map(|p| p.raw.trim().to_ascii_uppercase())
        .unwrap_or_else(|| "S".into());
    let anim_id = parse_first_int(sd.param("anim").map(|p| p.raw.as_str()))
        .or_else(|| {
            sd.controllers
                .iter()
                .find(|c| c.kind.eq_ignore_ascii_case("changeanim"))
                .and_then(|c| parse_first_int(c.raw("value")))
        })
        .unwrap_or(sd.id);

    let (startup, active, recovery, total, max_reach_px) = def
        .anims
        .get(&anim_id)
        .map(analyze_anim)
        .unwrap_or((0, 0, 0, 0, 0.0));

    let hitdef = sd
        .controllers
        .iter()
        .find(|c| c.kind.eq_ignore_ascii_case("hitdef"));
    let damage = hitdef
        .and_then(|c| parse_first_int(c.raw("damage")))
        .unwrap_or(0);
    let guard_flag = hitdef
        .and_then(|c| c.raw("guardflag"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "-".into());
    let hit_time = hitdef.and_then(|c| parse_first_int(c.raw("ground.hittime")));
    let guard_time = hitdef
        .and_then(|c| parse_first_int(c.raw("guard.hittime")))
        .or(hit_time);

    let hit_adv = hit_time.map(|ht| ht - recovery);
    let block_adv = guard_time.map(|gt| gt - recovery);

    Some(MoveFrameData {
        state: sd.id,
        anim: anim_id,
        state_type,
        startup,
        active,
        recovery,
        total,
        damage,
        guard_flag,
        hit_adv,
        block_adv,
        max_reach_px: (max_reach_px * 10.0).round() / 10.0,
    })
}

pub fn extract(spec: &str) -> Result<FighterFrameData, String> {
    let def = load_fighter(spec)?;
    let mut ids: Vec<i32> = def.states.keys().copied().collect();
    ids.sort_unstable();
    let mut moves = Vec::new();
    for id in ids {
        if id < 0 {
            continue;
        }
        if let Some(sd) = def.states.get(&id) {
            if let Some(row) = extract_state(&def, sd) {
                moves.push(row);
            }
        }
    }
    Ok(FighterFrameData {
        name: if def.display_name.is_empty() {
            def.name.clone()
        } else {
            def.display_name.clone()
        },
        moves,
    })
}

fn fmt_adv(v: Option<i32>) -> String {
    match v {
        Some(n) if n > 0 => format!("+{n}"),
        Some(n) => format!("{n}"),
        None => "-".into(),
    }
}

pub fn format_markdown(fd: &FighterFrameData) -> String {
    let mut out = format!("# Frame Data — {}\n\n", fd.name);
    out.push_str("| State | Type | Startup | Active | Recovery | Total | Damage | Guard | On Hit | On Block | Reach |\n");
    out.push_str("|---:|:---:|---:|---:|---:|---:|---:|:---:|---:|---:|---:|\n");
    for m in &fd.moves {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.0}px |\n",
            m.state,
            m.state_type,
            m.startup,
            m.active,
            m.recovery,
            m.total,
            m.damage,
            m.guard_flag,
            fmt_adv(m.hit_adv),
            fmt_adv(m.block_adv),
            m.max_reach_px
        ));
    }
    out
}

pub fn format_text(fd: &FighterFrameData) -> String {
    let mut out = format!(
        "Frame Data: {} ({} attack states)\n",
        fd.name,
        fd.moves.len()
    );
    out.push_str("STATE  TYPE  START  ACT  REC  TOT   DMG  GRD   HIT   BLK  REACH\n");
    for m in &fd.moves {
        out.push_str(&format!(
            "{:>5}  {:>4}  {:>5}  {:>3}  {:>3}  {:>3}  {:>4}  {:>3}  {:>4}  {:>4}  {:>4.0}px\n",
            m.state,
            m.state_type,
            m.startup,
            m.active,
            m.recovery,
            m.total,
            m.damage,
            m.guard_flag,
            fmt_adv(m.hit_adv),
            fmt_adv(m.block_adv),
            m.max_reach_px
        ));
    }
    out
}

pub fn to_json(fd: &FighterFrameData) -> Value {
    let moves: Vec<Value> = fd.moves.iter().map(|m| json!({
        "state": m.state, "anim": m.anim, "state_type": m.state_type,
        "startup": m.startup, "active": m.active, "recovery": m.recovery,
        "total": m.total, "damage": m.damage, "guard_flag": m.guard_flag,
        "hit_adv": m.hit_adv, "block_adv": m.block_adv, "max_reach_px": m.max_reach_px,
    })).collect();
    json!({ "name": fd.name, "moves": moves })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_frame_data_for_native_fighter_kaito() {
        let fd = extract("kaito").expect("extract kaito frame data");
        assert!(!fd.moves.is_empty(), "kaito must expose attacking states");
        assert!(
            fd.moves.iter().any(|m| m.startup > 0 && m.active > 0),
            "at least one attack must have measurable startup and active frames"
        );
        let md = format_markdown(&fd);
        assert!(md.contains("| State | Type | Startup |"));
        let json = serde_json::to_string(&to_json(&fd)).expect("serialize json");
        assert!(json.contains("\"startup\""));
    }
}
