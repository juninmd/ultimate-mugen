//! MUGEN command (CMD) parsing and backward matching against the input history.
use crate::input::{button_bit, rel_dirs, InputHistory};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CmdSymbol {
    pub release: bool,
    pub hold: bool,
    pub four: bool,
    pub strict: bool,
    pub neutral: bool,
    pub charge: i32,
    pub dir: u32,
    pub btns: u32,
}

#[derive(Clone, Debug)]
pub struct CommandDef {
    pub name: String,
    pub key: std::sync::Arc<str>,
    /** source text of the `command =` line (shown in the move list) */
    pub raw: String,
    pub steps: Vec<CmdSymbol>,
    pub time: i32,
    pub buffer_time: i32,
    pub pause_buffer: i32,
}

fn dir_bits(s: &str) -> Option<u32> {
    Some(match s {
        "U" => 1,
        "D" => 2,
        "B" => 4,
        "F" => 8,
        "UB" => 5,
        "UF" => 9,
        "DB" => 6,
        "DF" => 10,
        _ => return None,
    })
}

pub fn parse_command(name: &str, s: &str, time: i32, buffer_time: i32) -> CommandDef {
    let mut steps = Vec::with_capacity(4);
    for part in s.trim().trim_matches('"').split(',') {
        let mut p = part.trim();
        if p.is_empty() {
            continue;
        }
        let mut sym = CmdSymbol::default();
        loop {
            if let Some(r) = p.strip_prefix('>') {
                sym.strict = true;
                p = r.trim();
            } else if let Some(r) = p.strip_prefix('/') {
                sym.hold = true;
                p = r.trim();
            } else if let Some(r) = p.strip_prefix('$') {
                sym.four = true;
                p = r.trim();
            } else if let Some(r) = p.strip_prefix('~') {
                sym.release = true;
                let r = r.trim_start();
                let n: String = r.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !n.is_empty() {
                    sym.charge = n.parse().unwrap_or(0);
                }
                p = r[n.len()..].trim();
            } else {
                break;
            }
        }
        for tok in p.split('+') {
            let mut t = tok.trim();
            while let Some(c) = t.chars().next() {
                match c {
                    '/' => sym.hold = true,
                    '$' => sym.four = true,
                    '>' => sym.strict = true,
                    '~' => sym.release = true,
                    _ => break,
                }
                t = &t[1..];
            }
            if t == "N" {
                sym.neutral = true;
            } else if let Some(d) = dir_bits(t) {
                sym.dir |= d;
            } else if let Some(b) = button_bit(t) {
                sym.btns |= b;
            }
        }
        steps.push(sym);
    }
    CommandDef {
        name: name.to_string(),
        key: name.into(),
        raw: s.trim().to_string(),
        steps,
        time: if time > 0 { time } else { 15 },
        buffer_time: buffer_time.max(1),
        pause_buffer: 0,
    }
}

#[inline]
fn held(h: &InputHistory, t: i64, s: &CmdSymbol) -> bool {
    let bits = h.at(t);
    if bits & s.btns != s.btns {
        return false;
    }
    if s.neutral && bits & 15 != 0 {
        return false;
    }
    if s.dir != 0 {
        let rel = rel_dirs(bits, h.face_at(t));
        let ok = if s.four {
            rel & s.dir == s.dir
        } else {
            rel == s.dir
        };
        if !ok {
            return false;
        }
    }
    true
}
fn is_state_step(s: &CmdSymbol) -> bool {
    s.hold || (s.neutral && s.btns == 0 && s.dir == 0)
}
fn held_dur(h: &InputHistory, t: i64, s: &CmdSymbol) -> i32 {
    let mut n = 0;
    while t - (n as i64) >= 0 && n < 240 && held(h, t - n as i64, s) {
        n += 1;
    }
    n
}
fn fires(h: &InputHistory, t: i64, s: &CmdSymbol) -> bool {
    if is_state_step(s) {
        return held(h, t, s);
    }
    if s.release {
        if held(h, t, s) || !held(h, t - 1, s) {
            return false;
        }
        return s.charge == 0 || held_dur(h, t - 1, s) >= s.charge;
    }
    if s.btns != 0 {
        if !held(h, t, s) {
            return false;
        }
        let (now, prev) = (h.at(t), h.at(t - 1));
        if s.btns & now & !prev == 0 {
            return false;
        }
        let (p2, p3) = (h.at(t - 2), h.at(t - 3));
        let pressed_recently = (now & !prev) | (prev & !p2) | (p2 & !p3);
        if s.btns & !pressed_recently != 0 {
            return false;
        }
        return true;
    }
    held(h, t, s) && !held(h, t - 1, s)
}

pub fn match_command(h: &InputHistory, c: &CommandDef) -> bool {
    let n = c.steps.len();
    if n == 0 {
        return false;
    }
    let tt = h.tick;
    let last = &c.steps[n - 1];
    let mut t_end = tt;
    let limit_buf = tt - c.buffer_time as i64;
    while t_end > limit_buf && t_end >= 0 {
        if fires(h, t_end, last) {
            if n == 1 {
                return true;
            }
            let limit = t_end - c.time as i64;
            let mut t = t_end;
            let mut ok = true;
            let lower_bound = limit.max(0);
            for i in (0..n - 1).rev() {
                let s = &c.steps[i];
                let strict_next = c.steps[i + 1].strict;
                let mut found = -1i64;
                let mut x = t - if is_state_step(s) || s.release { 0 } else { 1 };
                while x >= lower_bound {
                    if fires(h, x, s) {
                        found = x;
                        break;
                    }
                    if strict_next && x < t && h.at(x) != h.at(x - 1) && !is_state_step(s) {
                        break;
                    }
                    x -= 1;
                }
                if found < 0 {
                    ok = false;
                    break;
                }
                t = found;
            }
            if ok {
                return true;
            }
        }
        t_end -= 1;
    }
    false
}

#[derive(Clone, Default)]
pub struct CommandState {
    pub active: std::collections::HashSet<std::sync::Arc<str>>,
    pause_hold: std::collections::HashMap<std::sync::Arc<str>, i32>,
}
impl CommandState {
    pub fn update(&mut self, h: &InputHistory, cmds: &[CommandDef], paused: bool) {
        self.active.clear();
        if !self.pause_hold.is_empty() {
            for v in self.pause_hold.values_mut() {
                *v -= 1;
            }
            self.pause_hold.retain(|_, v| *v > 0);
        }
        for c in cmds {
            if self.active.contains(c.name.as_str()) {
                continue;
            }
            if match_command(h, c) {
                if paused && c.pause_buffer > 0 {
                    self.pause_hold.insert(c.key.clone(), c.pause_buffer);
                }
                self.active.insert(c.key.clone());
            } else if self.pause_hold.contains_key(c.name.as_str()) {
                self.active.insert(c.key.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::k::*;
    fn feed(seq: &[u32], f: i32) -> InputHistory {
        let mut h = InputHistory::default();
        for b in seq {
            h.push(*b, f);
        }
        h
    }
    #[test]
    fn qcf() {
        assert!(match_command(
            &feed(&[0, D, D, D | R, R, R | X], 1),
            &parse_command("q", "~D, DF, F, x", 15, 1)
        ));
    }
    #[test]
    fn wrong() {
        assert!(!match_command(
            &feed(&[0, R, D | R, D, X], 1),
            &parse_command("q", "~D, DF, F, x", 15, 1)
        ));
    }
    #[test]
    fn left() {
        assert!(match_command(
            &feed(&[0, D, D, D | L, L, L | X], -1),
            &parse_command("q", "~D, DF, F, x", 15, 1)
        ));
    }
    #[test]
    fn dash_hold() {
        let h = feed(&[0, R, 0, R], 1);
        assert!(match_command(&h, &parse_command("f", "F, F", 10, 1)));
        assert!(match_command(&h, &parse_command("h", "/$F", 1, 1)));
    }
    #[test]
    fn combo() {
        assert!(match_command(
            &feed(&[0, A, A | B], 1),
            &parse_command("ab", "a+b", 15, 1)
        ));
        let mut v = vec![0];
        v.extend(std::iter::repeat_n(L, 40));
        v.push(R);
        v.push(R | X);
        assert!(match_command(
            &feed(&v, 1),
            &parse_command("c", "~30$B, F, x", 20, 1)
        ));
    }
    #[test]
    fn super_motion() {
        let q = [D, D, D | R, D | R, R, R];
        let mut v = vec![0];
        v.extend(q);
        v.push(0);
        v.extend(&q[..5]);
        v.push(R | X);
        assert!(match_command(
            &feed(&v, 1),
            &parse_command("s", "~D, DF, F, D, DF, F, x", 20, 1)
        ));
    }
    #[test]
    fn neutral_step() {
        assert!(match_command(
            &feed(&[0, R, 0, 0, X], 1),
            &parse_command("n", "N, x", 15, 1)
        ));
        let mut held_forward = vec![0];
        held_forward.extend(std::iter::repeat_n(R, 20));
        held_forward.push(R | X);
        assert!(!match_command(
            &feed(&held_forward, 1),
            &parse_command("n", "N, x", 15, 1)
        ));
        assert!(match_command(
            &feed(&[0, 0, X], 1),
            &parse_command("n", "N, x", 15, 1)
        ));
        assert!(match_command(
            &feed(&[0, D, D | R, 0, X], 1),
            &parse_command("n", "~D, N, x", 15, 1)
        ));
        assert!(match_command(
            &feed(&[0, 0], 1),
            &parse_command("n", "N", 1, 1)
        ));
        assert!(!match_command(
            &feed(&[0, U], 1),
            &parse_command("n", "N", 1, 1)
        ));
    }
    #[test]
    fn neutral_with_button_in_one_step() {
        assert!(match_command(
            &feed(&[0, 0, X], 1),
            &parse_command("n", "N+x", 1, 1)
        ));
        assert!(!match_command(
            &feed(&[0, 0, R | X], 1),
            &parse_command("n", "N+x", 1, 1)
        ));
    }
    #[test]
    fn a_button_that_never_gets_released_stops_matching_until_the_history_clears() {
        let mut h = feed(&[0, D, D, D | R, D | R, R, R | X], 1);
        let qcf = parse_command("q", "~D, DF, F, x", 15, 1);
        let mut st = CommandState::default();
        st.update(&h, std::slice::from_ref(&qcf), false);
        assert!(st.active.contains("q"));
        // the key-up was lost (window lost focus while x was held): no press edge ever happens again
        for _ in 0..30 { h.push(R | X, 1); st.update(&h, std::slice::from_ref(&qcf), false); }
        assert!(!st.active.contains("q"), "a permanently held button cannot fire the same command twice");
        // releasing everything and rolling the motion again works, which is what the shell restore must allow
        for _ in 0..3 { h.push(0, 1); }
        for b in [D, D, D | R, D | R, R, R | X] { h.push(b, 1); }
        st.update(&h, std::slice::from_ref(&qcf), false);
        assert!(st.active.contains("q"));
    }

    fn pausing_state(c: &CommandDef, h: &InputHistory, ticks: usize) -> Vec<bool> {
        let mut st = CommandState::default();
        let mut h = h.clone();
        let mut out = vec![];
        st.update(&h, std::slice::from_ref(c), true);
        out.push(st.active.contains(c.name.as_str()));
        for _ in 0..ticks {
            h.push(0, 1);
            st.update(&h, std::slice::from_ref(c), true);
            out.push(st.active.contains(c.name.as_str()));
        }
        out
    }
    #[test]
    fn pause_buffer_keeps_command_while_paused() {
        let h = feed(&[0, X], 1);
        let mut c = parse_command("p", "x", 15, 1);
        assert_eq!(pausing_state(&c, &h, 4), [true, false, false, false, false]);
        c.pause_buffer = 3;
        assert_eq!(
            pausing_state(&c, &h, 5),
            [true, true, true, false, false, false]
        );
    }
    #[test]
    fn pause_buffer_only_arms_during_pause() {
        let mut c = parse_command("p", "x", 15, 1);
        c.pause_buffer = 3;
        let mut st = CommandState::default();
        let mut h = feed(&[0, X], 1);
        st.update(&h, std::slice::from_ref(&c), false);
        assert!(st.active.contains("p"));
        h.push(0, 1);
        st.update(&h, std::slice::from_ref(&c), false);
        assert!(!st.active.contains("p"));
    }
}
