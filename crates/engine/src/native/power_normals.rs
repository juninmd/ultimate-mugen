use super::types::{HitSpec, NativeChar, NativeMove};

const DAMAGE_GAIN: f32 = 1.3;
const KNOCKBACK_GAIN: f32 = 1.25;
const EXTRA_HITSTUN: i32 = 4;
const EXTRA_HITSTOP: i32 = 2;
const MAX_LEVEL: &str = "heavy";

struct Spec {
    id: &'static str,
    button: &'static str,
    crouch: bool,
    sources: [&'static str; 2],
    label: &'static str,
}

const SPECS: [Spec; 4] = [
    Spec { id: "power_punch", button: "z", crouch: false, sources: ["y", "x"], label: "Max" },
    Spec { id: "power_kick", button: "c", crouch: false, sources: ["b", "a"], label: "Max" },
    Spec { id: "crouch_power_punch", button: "z", crouch: true, sources: ["y", "x"], label: "Max" },
    Spec { id: "crouch_power_kick", button: "c", crouch: true, sources: ["b", "a"], label: "Max" },
];

fn is_crouch(m: &NativeMove) -> bool {
    m.when.as_deref() == Some("crouch")
}

fn is_ground_normal(m: &NativeMove, crouch: bool) -> bool {
    m.kind == "normal" && m.throw.is_none() && !m.hits.is_empty() && matches!(m.when.as_deref(), None | Some("stand") | Some("crouch")) && is_crouch(m) == crouch
}

fn source<'a>(nc: &'a NativeChar, spec: &Spec) -> Option<&'a NativeMove> {
    spec.sources.iter().find_map(|button| nc.moves.iter().find(|m| m.input == *button && is_ground_normal(m, spec.crouch)))
}

fn empower(hit: &HitSpec) -> HitSpec {
    let mut out = hit.clone();
    out.damage = (hit.damage * DAMAGE_GAIN).round();
    out.guard_damage = hit.guard_damage.map(|g| (g * DAMAGE_GAIN).round());
    out.level = Some(MAX_LEVEL.into());
    out.hitstun = hit.hitstun.map(|s| s + EXTRA_HITSTUN);
    out.hitstop = hit.hitstop.map(|s| s + EXTRA_HITSTOP);
    out.knockback = hit.knockback.map(|k| [k[0] * KNOCKBACK_GAIN, k[1] * KNOCKBACK_GAIN]);
    out
}

fn build(base: &NativeMove, spec: &Spec) -> NativeMove {
    let mut out = base.clone();
    out.id = spec.id.into();
    out.name = format!("{} {}", base.name, spec.label);
    out.input = spec.button.into();
    out.hits = base.hits.iter().map(empower).collect();
    out.simple = None;
    out
}

fn taken(nc: &NativeChar) -> bool {
    nc.moves.iter().any(|m| m.input == "z" || m.input == "c")
}

pub(super) fn with_power_normals(mut nc: NativeChar) -> NativeChar {
    if taken(&nc) {
        return nc;
    }
    let added: Vec<(String, NativeMove)> = SPECS.iter().filter_map(|spec| source(&nc, spec).map(|base| (base.id.clone(), build(base, spec)))).collect();
    for (base_id, power) in added {
        for m in nc.moves.iter_mut() {
            if let Some(cancels) = m.cancels.as_mut() {
                if cancels.contains(&base_id) {
                    cancels.push(power.id.clone());
                }
            }
        }
        nc.moves.push(power);
    }
    nc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn akio() -> NativeChar {
        serde_json::from_str(include_str!("../../../../assets/chars/akio.json")).unwrap()
    }

    #[test]
    fn every_roster_fighter_gets_both_ground_power_normals() {
        for entry in std::fs::read_dir("../../assets/chars").unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "json") || path.file_stem().is_some_and(|s| s == "index") {
                continue;
            }
            let nc: NativeChar = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let out = with_power_normals(nc);
            for id in ["power_punch", "power_kick", "crouch_power_punch", "crouch_power_kick"] {
                assert!(out.moves.iter().any(|m| m.id == id), "{} lacks {id}", out.id);
            }
        }
    }

    #[test]
    fn power_normals_use_z_and_c_and_hit_harder_than_their_base() {
        let base = akio();
        let out = with_power_normals(base.clone());
        let punch = out.moves.iter().find(|m| m.id == "power_punch").unwrap();
        let strong = base.moves.iter().find(|m| m.id == "strong").unwrap();
        assert_eq!((punch.input.as_str(), punch.anim.as_str()), ("z", strong.anim.as_str()));
        assert!(punch.hits[0].damage > strong.hits[0].damage);
        assert_eq!(punch.hits[0].level.as_deref(), Some("heavy"));
        let kick = out.moves.iter().find(|m| m.id == "crouch_power_kick").unwrap();
        assert_eq!((kick.input.as_str(), kick.when.as_deref()), ("c", Some("crouch")));
    }

    #[test]
    fn explicit_cancels_into_the_base_also_reach_the_power_version() {
        let out = with_power_normals(akio());
        let jab = out.moves.iter().find(|m| m.id == "jab").unwrap();
        assert!(jab.cancels.as_ref().unwrap().contains(&"power_punch".to_string()));
    }

    #[test]
    fn fighters_with_their_own_z_or_c_are_left_alone() {
        let once = with_power_normals(akio());
        let twice = with_power_normals(once.clone());
        assert_eq!(once.moves.len(), twice.moves.len());
    }
}
