use super::types::NativeMove;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Ground,
    Crouch,
    Air,
    Special,
    Super,
}

struct Entry {
    id: String,
    sid: i32,
    strength: u8,
    lane: Lane,
    explicit: bool,
    targets: Vec<String>,
    projectiles: i32,
    air: bool,
}

pub(super) struct ChainPlan {
    entries: Vec<Entry>,
}

fn lane_of(m: &NativeMove) -> Lane {
    match (m.kind.as_str(), m.when.as_deref().unwrap_or("stand")) {
        ("super", _) => Lane::Super,
        ("normal", "crouch") => Lane::Crouch,
        ("normal", "air") => Lane::Air,
        ("normal", _) => Lane::Ground,
        _ => Lane::Special,
    }
}

fn strength_of(m: &NativeMove) -> u8 {
    let level = |l: Option<&str>| match l {
        Some("light") => 1,
        Some("heavy") => 3,
        _ => 2,
    };
    let from_hits = m.hits.iter().map(|h| level(h.level.as_deref())).max();
    let from_projectiles = m.projectile.as_ref().map(|p| {
        p.to_vec()
            .iter()
            .map(|p| level(p.level.as_deref()))
            .max()
            .unwrap_or(2)
    });
    from_hits.or(from_projectiles).unwrap_or(2)
}

impl ChainPlan {
    pub(super) fn new<'a>(moves: impl Iterator<Item = &'a NativeMove>) -> Self {
        let mut next: HashMap<u8, i32> = HashMap::new();
        let entries = moves
            .map(|m| {
                let lane = lane_of(m);
                let base = match lane {
                    Lane::Ground => 200,
                    Lane::Crouch => 400,
                    Lane::Air => 600,
                    Lane::Special => 1000,
                    Lane::Super => 3000,
                };
                let counter = next.entry(lane as u8).or_insert(base);
                let sid = *counter;
                *counter += 1;
                Entry {
                    id: m.id.clone(),
                    sid,
                    strength: strength_of(m),
                    lane,
                    explicit: m.cancels.is_some(),
                    targets: m.cancels.clone().unwrap_or_default(),
                    projectiles: m.projectile.as_ref().map_or(0, |p| p.to_vec().len() as i32),
                    air: m.when.as_deref() == Some("air"),
                }
            })
            .collect();
        ChainPlan { entries }
    }

    pub(super) fn sid(&self, index: usize) -> i32 {
        self.entries[index].sid
    }

    fn sources(&self, index: usize) -> Vec<i32> {
        let target = &self.entries[index];
        let mut out: Vec<i32> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(i, s)| {
                if *i == index {
                    return false;
                }
                if s.explicit {
                    return s.targets.iter().any(|t| *t == target.id);
                }
                let both_ground = matches!(s.lane, Lane::Ground | Lane::Crouch)
                    && matches!(target.lane, Lane::Ground | Lane::Crouch);
                let both_air = s.lane == Lane::Air && target.lane == Lane::Air;
                (both_ground || both_air) && (s.strength, s.sid) < (target.strength, target.sid)
            })
            .map(|(_, s)| s.sid)
            .collect();
        out.sort();
        out
    }

    /// Air specials may also start from the launcher states that explicitly cancel into them (uppercut -> dive).
    pub(super) fn air_entry_states(&self, index: usize) -> String {
        if !self.entries[index].air {
            return String::new();
        }
        self.sources(index)
            .iter()
            .filter(|s| (1000..3000).contains(*s) || (200..600).contains(*s))
            .map(|s| format!(" || stateno = {s}"))
            .collect()
    }

    fn contact_clause(&self, sid: i32) -> String {
        let projectiles = self
            .entries
            .iter()
            .find(|e| e.sid == sid)
            .map_or(0, |e| e.projectiles);
        let mut contact = String::from("movecontact");
        for pid in sid..sid + projectiles {
            contact += &format!(" || (ProjContactTime({pid}) >= 0 && ProjContactTime({pid}) < 45)");
        }
        format!("(stateno = {sid} && ({contact}))")
    }

    fn after_contact(&self, ids: &[i32]) -> String {
        ids.iter()
            .map(|s| self.contact_clause(*s))
            .collect::<Vec<_>>()
            .join(" || ")
    }

    pub(super) fn triggers(&self, index: usize) -> String {
        let target = &self.entries[index];
        let sources = self.sources(index);
        match target.lane {
            Lane::Ground | Lane::Crouch | Lane::Air => {
                if sources.is_empty() {
                    String::new()
                } else {
                    format!(
                        "
trigger2 = time > 2 && ({})",
                        self.after_contact(&sources)
                    )
                }
            }
            Lane::Special => {
                let mut out = String::from(
                    "
trigger2 = movecontact && stateno = [200, 699]",
                );
                let specials: Vec<i32> = sources
                    .iter()
                    .copied()
                    .filter(|s| (1000..3000).contains(s))
                    .collect();
                if !specials.is_empty() {
                    out += &format!(
                        "
trigger3 = time > 6 && ({})",
                        self.after_contact(&specials)
                    );
                }
                out
            }
            Lane::Super => {
                let mut out = format!(
                    "
trigger2 = movecontact && stateno = [200, 2999] && stateno != {}",
                    target.sid
                );
                let shooters: Vec<i32> = self
                    .entries
                    .iter()
                    .filter(|e| e.lane == Lane::Special && e.projectiles > 0)
                    .map(|e| e.sid)
                    .collect();
                if !shooters.is_empty() {
                    out += &format!(
                        "
trigger3 = {}",
                        self.after_contact(&shooters)
                    );
                }
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mv(value: serde_json::Value) -> NativeMove {
        serde_json::from_value(value).unwrap()
    }
    fn normal(id: &str, level: &str, when: &str) -> NativeMove {
        mv(
            json!({"id": id, "name": id, "input": "x", "kind": "normal", "anim": "a", "when": when, "hits": [{"at": 1, "damage": 10, "level": level}]}),
        )
    }

    #[test]
    fn gatling_only_climbs_in_strength_so_chains_cannot_loop() {
        let moves = [
            normal("jab", "light", "stand"),
            normal("kick", "medium", "stand"),
            normal("heavy", "heavy", "stand"),
            normal("sweep", "heavy", "crouch"),
        ];
        let plan = ChainPlan::new(moves.iter());
        assert!(
            plan.triggers(0).is_empty(),
            "the lightest normal accepts no gatling source"
        );
        assert!(plan.triggers(1).contains("stateno = 200"));
        assert!(!plan.triggers(1).contains("stateno = 201"));
        assert!(
            plan.triggers(2).contains("stateno = 200")
                && plan.triggers(2).contains("stateno = 201")
        );
        assert!(
            plan.triggers(3).contains("stateno = 202"),
            "ground normals link into crouching ones"
        );
    }

    #[test]
    fn explicit_cancels_name_the_moves_they_feed() {
        let mut jab = normal("jab", "light", "stand");
        jab.cancels = Some(vec!["rush".into()]);
        let rush = mv(
            json!({"id": "rush", "name": "rush", "input": "~D, DF, F, y", "kind": "special", "anim": "a"}),
        );
        let follow = mv(
            json!({"id": "follow", "name": "follow", "input": "~D, DF, F, x", "kind": "special", "anim": "a", "cancels": ["rush"]}),
        );
        let plan = ChainPlan::new([jab, rush, follow].iter());
        assert!(
            plan.triggers(1).contains("stateno = 1001"),
            "special that lists rush may cancel into it"
        );
        assert!(
            !plan.triggers(2).contains("trigger3"),
            "nothing cancels into follow"
        );
    }
}
