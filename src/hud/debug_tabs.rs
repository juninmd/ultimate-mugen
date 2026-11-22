use crate::session::debug_ctrl::DebugTab;
use ume_engine::fighter::Fighter;
use ume_engine::flags::asf;
use ume_engine::matchs::Match;

pub(super) fn tab_rows(m: &Match, slot: usize, tab: DebugTab) -> Vec<String> {
    let Some(f) = m.fighters.get(slot) else {
        return vec![];
    };
    match tab {
        DebugTab::Overview => vec![
            format!(
                "state {} (prev {}) t{} form {}",
                f.state_no, f.prev_state_no, f.state_time, f.form
            ),
            format!(
                "anim {} elem {} t{} AnimTime {}",
                f.anim_no,
                f.elem + 1,
                f.elem_time,
                f.anim_time_trig()
            ),
            format!(
                "{}/{} ctrl:{} alive:{} hitstop:{}",
                f.state_type,
                f.move_type,
                u8::from(f.ctrl),
                u8::from(f.alive),
                f.hit_pause
            ),
            format!(
                "life {:.0}/{:.0} pow {:.0} jug {}",
                f.life,
                f.life_max,
                f.power,
                f.juggle_ledger.iter().map(|(_, p)| *p).sum::<i32>()
            ),
            format!(
                "pos ({:.1},{:.1}) vel ({:.2},{:.2})",
                f.pos.x, f.pos.y, f.vel.x, f.vel.y
            ),
            format!(
                "dmg*{:.2} def*{:.2} combo:{} tgts:{}",
                f.attack_mul,
                f.defence_mul,
                f.ghv.hitcount,
                f.targets.len()
            ),
        ],
        DebugTab::VarsMaps => {
            let vars: Vec<String> = f
                .vars
                .iter()
                .enumerate()
                .filter(|(_, v)| **v != 0)
                .take(6)
                .map(|(i, v)| format!("{i}={v}"))
                .collect();
            let fvars: Vec<String> = f
                .fvars
                .iter()
                .enumerate()
                .filter(|(_, v)| **v != 0.0)
                .take(5)
                .map(|(i, v)| format!("{i}={v:.2}"))
                .collect();
            let mut maps: Vec<String> = f
                .ikemen
                .map
                .iter()
                .take(5)
                .map(|(k, v)| format!("{k}={v:.2}"))
                .collect();
            if maps.is_empty() {
                maps.push("-".into());
            }
            let tmaps: Vec<String> = m.team_maps[f.side.min(1)]
                .iter()
                .take(4)
                .map(|(k, v)| format!("{k}={v:.1}"))
                .collect();
            vec![
                format!(
                    "var: {}",
                    if vars.is_empty() {
                        "-".into()
                    } else {
                        vars.join(" ")
                    }
                ),
                format!(
                    "fvar: {}",
                    if fvars.is_empty() {
                        "-".into()
                    } else {
                        fvars.join(" ")
                    }
                ),
                format!("sysvar: {:?}", f.sysvars),
                format!("map: {}", maps.join(" ")),
                format!(
                    "teamMap: {}",
                    if tmaps.is_empty() {
                        "-".into()
                    } else {
                        tmaps.join(" ")
                    }
                ),
                format!("asserts: {}", active_assert_summary(f)),
            ]
        }
        DebugTab::Clipboard => {
            let mut out = Vec::new();
            if f.clipboard.is_empty() {
                out.push("clipboard: (empty)".into());
            } else {
                for line in f.clipboard.iter().rev().take(4).rev() {
                    out.push(format!("clip: {line}"));
                }
            }
            let hd = f.hitdef.as_ref().map_or_else(
                || "hitdef: -".to_string(),
                |h| {
                    format!(
                        "hitdef dmg {:.0} p{},{} stun {}",
                        h.damage, h.pause_p1, h.pause_p2, h.ground_hit_time
                    )
                },
            );
            out.push(hd);
            let h_list: Vec<String> = m
                .fighters
                .iter()
                .filter(|h| !h.destroyed && h.is_helper && h.root == f.id)
                .take(4)
                .map(|h| format!("#{}@S{}", h.helper_id, h.state_no))
                .collect();
            out.push(format!(
                "helpers: {}",
                if h_list.is_empty() {
                    "-".into()
                } else {
                    h_list.join(" ")
                }
            ));
            out
        }
        DebugTab::Console => {
            let mut out = vec![
                format!(
                    "pause: {:?} slow_mo: {}",
                    m.pause.as_ref().map(|p| (p.time, p.is_super)),
                    m.slow_mo
                ),
                format!(
                    "texts: {} warnings: {} dropped: {}",
                    m.texts.len(),
                    m.warnings.len(),
                    m.console.dropped()
                ),
            ];
            for w in m.warnings.iter().rev().take(3) {
                out.push(format!("warn: {w}"));
            }
            out
        }
    }
}

fn active_assert_summary(f: &Fighter) -> String {
    let mut tags = Vec::new();
    for (name, bit) in [
        ("invis", asf::INVISIBLE),
        ("noko", asf::NOKO),
        ("unguard", asf::UNGUARDABLE),
        ("nobar", asf::NOLIFEBARDISPLAY),
        ("noturn", asf::NOAUTOTURN),
        ("noinput", asf::NOINPUT),
    ] {
        if f.has_assert(bit) {
            tags.push(name);
        }
    }
    if tags.is_empty() {
        "-".into()
    } else {
        tags.join(",")
    }
}
