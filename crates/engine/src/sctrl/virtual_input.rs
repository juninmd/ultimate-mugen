use crate::chardef::Controller;
use crate::fighter::Fighter;
use crate::input::k;
use crate::matchs::Match;

const MAX_ASSERTS: usize = 64;

impl Fighter {
    pub fn take_asserted_inputs(&mut self) -> u32 {
        let mut bits = 0;
        for (b, left) in self.input_asserts.iter_mut() {
            bits |= *b;
            *left -= 1;
        }
        self.input_asserts.retain(|(_, left)| *left > 0);
        bits
    }

    pub fn apply_command_asserts(&mut self) {
        for (name, left) in self.command_asserts.iter_mut() {
            self.cmd.active.insert(name.clone());
            *left -= 1;
        }
        self.command_asserts.retain(|(_, left)| *left > 0);
    }
}

fn input_owner(m: &Match, i: usize, sc: &Controller) -> Option<usize> {
    let target = if sc.has("redirectid", 0) { m.idx(m.num(i, sc, "redirectid", 0.0, 0) as u32)? } else { i };
    let f = &m.fighters[target];
    if f.is_helper && f.key_ctrl { m.idx(f.root) } else { Some(target) }
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').trim().to_string()
}

pub fn input_flags(raw: &str, facing: f32) -> u32 {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .map(|t| match t {
            "U" => k::U,
            "D" => k::D,
            "L" => k::L,
            "R" => k::R,
            "F" => if facing >= 0.0 { k::R } else { k::L },
            "B" => if facing >= 0.0 { k::L } else { k::R },
            other => crate::input::button_bit(other).unwrap_or(0),
        })
        .fold(0, |acc, b| acc | b)
}

pub fn assert_command(m: &mut Match, i: usize, sc: &Controller) {
    let Some(name) = sc.raw("name").map(unquote).filter(|n| !n.is_empty()) else { return };
    let time = (m.num(i, sc, "buffertime", 1.0, 0) as i32).max(1);
    let Some(t) = input_owner(m, i, sc) else { return };
    let f = &mut m.fighters[t];
    f.cmd.active.insert(name.clone());
    f.command_asserts.retain(|(n, _)| *n != name);
    if time > 1 && f.command_asserts.len() < MAX_ASSERTS {
        f.command_asserts.push((name, time - 1));
    }
}

pub fn assert_input(m: &mut Match, i: usize, sc: &Controller) {
    let time = (m.num(i, sc, "time", 1.0, 0) as i32).max(1);
    let Some(t) = input_owner(m, i, sc) else { return };
    let facing = m.fighters[t].facing;
    let bits = ["flag", "flag2", "flag3", "flag4"].iter().filter_map(|key| sc.raw(key)).fold(0, |acc, raw| acc | input_flags(raw, facing));
    if bits == 0 {
        return;
    }
    let f = &mut m.fighters[t];
    f.history.or_current(bits);
    let already_active = std::mem::take(&mut f.cmd.active);
    let def = f.def.clone();
    f.cmd.update(&f.history, &def.commands);
    f.cmd.active.extend(already_active);
    if time > 1 && f.input_asserts.len() < MAX_ASSERTS {
        f.input_asserts.push((bits, time - 1));
    }
}

pub fn shift_input(m: &mut Match, i: usize, sc: &Controller) {
    let ticks = m.num(i, sc, "value", 0.0, 0) as i32;
    let Some(t) = input_owner(m, i, sc) else { return };
    m.fighters[t].history.shift(ticks);
}
