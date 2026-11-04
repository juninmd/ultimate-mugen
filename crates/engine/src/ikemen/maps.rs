use crate::chardef::Controller;
use crate::matchs::Match;

pub const MAP_MAX_KEYS: usize = 256;
pub const MAP_MAX_KEY_LEN: usize = 64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapStore {
    entries: Vec<(String, f32)>,
}

impl MapStore {
    pub fn get(&self, key: &str) -> f32 {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| *v).unwrap_or(0.0)
    }

    pub fn set(&mut self, key: &str, v: f32, add: bool) {
        if let Some((_, x)) = self.entries.iter_mut().find(|(k, _)| k == key) {
            *x = if add { *x + v } else { v };
        } else if self.entries.len() < MAP_MAX_KEYS && key.len() <= MAP_MAX_KEY_LEN {
            self.entries.push((key.to_string(), v));
        }
    }

    pub fn reset(&mut self, key: Option<&str>) {
        match key {
            Some(k) => self.entries.retain(|(name, _)| name != k),
            None => self.entries.clear(),
        }
    }

    pub fn len(&self) -> usize { self.entries.len() }

    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scope {
    Local,
    Team,
    Root,
    Parent,
}

fn map_key(sc: &Controller) -> Option<String> {
    let raw = sc.raw("map")?;
    let k = raw.trim().trim_matches('"').trim().to_ascii_lowercase();
    (!k.is_empty()).then_some(k)
}

fn split_kind(kind: &str) -> Option<(Scope, &str)> {
    for (prefix, scope) in [("team", Scope::Team), ("root", Scope::Root), ("parent", Scope::Parent)] {
        if let Some(op) = kind.strip_prefix(prefix) {
            if matches!(op, "mapset" | "mapadd" | "mapreset") { return Some((scope, op)); }
        }
    }
    matches!(kind, "mapset" | "mapadd" | "mapreset").then_some((Scope::Local, kind))
}

fn scope_owner(m: &Match, i: usize, scope: Scope) -> Option<usize> {
    match scope {
        Scope::Local => Some(i),
        Scope::Root => m.idx(m.fighters[i].root),
        Scope::Parent => m.fighters[i].parent.and_then(|p| m.idx(p)),
        Scope::Team => None,
    }
}

fn store_for(m: &mut Match, i: usize, scope: Scope) -> Option<&mut MapStore> {
    if scope == Scope::Team {
        let side = m.fighters[i].side;
        return m.team_maps.get_mut(side);
    }
    let owner = scope_owner(m, i, scope)?;
    Some(&mut m.fighters[owner].ikemen.map)
}

pub fn team_value(m: &Match, i: usize, key: &str) -> f32 {
    m.team_maps.get(m.fighters[i].side).map(|s| s.get(key)).unwrap_or(0.0)
}

fn run_root_var(m: &mut Match, i: usize, sc: &Controller) {
    let Some(root) = m.idx(m.fighters[i].root) else { return };
    crate::controllers::set_vars(m, i, sc, sc.kind == "rootvaradd", root);
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    let kind = sc.kind.as_str();
    if matches!(kind, "rootvarset" | "rootvaradd") {
        run_root_var(m, i, sc);
        return true;
    }
    let Some((scope, op)) = split_kind(kind) else { return false };
    let key = map_key(sc);
    match op {
        "mapreset" => {
            if let Some(store) = store_for(m, i, scope) { store.reset(key.as_deref()); }
        }
        _ => {
            let Some(key) = key else { return true };
            let v = m.num(i, sc, "value", 0.0, 0) as f32;
            let add = op == "mapadd" || m.num(i, sc, "type", 0.0, 0) as i32 == 1;
            if let Some(store) = store_for(m, i, scope) { store.set(&key, v, add); }
        }
    }
    true
}
