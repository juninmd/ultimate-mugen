use crate::chardef::{Param, StateDef};
use crate::expr::Expr;
use crate::cns::build_controller;
use std::collections::HashMap;
use ume_formats::ini::Section;

#[derive(Clone, Default)]
struct Scope { conds: Vec<String>, persistent: Option<i32>, ignore_hitpause: bool }

struct Reader<'a> { s: &'a [u8], p: usize, unsupported: Vec<String> }

impl<'a> Reader<'a> {
    fn skip(&mut self) {
        while self.p < self.s.len() {
            match self.s[self.p] {
                b'#' => while self.p < self.s.len() && self.s[self.p] != b'\n' { self.p += 1; },
                c if c.is_ascii_whitespace() || c == b';' => self.p += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> { self.s.get(self.p).copied() }

    fn word(&mut self) -> String {
        let st = self.p;
        while self.p < self.s.len() && (self.s[self.p].is_ascii_alphanumeric() || self.s[self.p] == b'_' || self.s[self.p] == b'.') { self.p += 1; }
        String::from_utf8_lossy(&self.s[st..self.p]).into_owned()
    }

    fn until(&mut self, stops: &[u8]) -> String {
        let st = self.p;
        let mut depth = 0i32;
        while self.p < self.s.len() {
            let c = self.s[self.p];
            match c {
                b'"' => { self.p += 1; while self.p < self.s.len() && self.s[self.p] != b'"' { self.p += 1; } }
                b'#' if depth == 0 => break,
                b'(' | b'[' => depth += 1,
                b')' | b']' if depth > 0 => depth -= 1,
                _ if depth == 0 && stops.contains(&c) => break,
                _ => {}
            }
            self.p += 1;
        }
        let out = String::from_utf8_lossy(&self.s[st..self.p]).trim().to_string();
        if self.p < self.s.len() && self.s[self.p] == b'#' { while self.p < self.s.len() && self.s[self.p] != b'\n' { self.p += 1; } }
        out
    }

    fn braced(&mut self) -> String {
        self.skip();
        if self.peek() != Some(b'{') { return String::new(); }
        self.p += 1;
        let st = self.p;
        let mut depth = 1;
        while self.p < self.s.len() {
            match self.s[self.p] {
                b'"' => { self.p += 1; while self.p < self.s.len() && self.s[self.p] != b'"' { self.p += 1; } }
                b'{' => depth += 1,
                b'}' => { depth -= 1; if depth == 0 { break; } }
                _ => {}
            }
            self.p += 1;
        }
        let out = String::from_utf8_lossy(&self.s[st..self.p.min(self.s.len())]).into_owned();
        self.p += 1;
        out
    }

    fn block_or_stmt(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        self.skip();
        if self.peek() == Some(b'{') {
            self.p += 1;
            self.stmts(sc, out, false);
            self.skip();
            if self.peek() == Some(b'}') { self.p += 1; }
        } else {
            self.stmt(sc, out);
        }
    }

    fn stmts(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>, in_switch: bool) {
        loop {
            self.skip();
            match self.peek() {
                None | Some(b'}') | Some(b'[') => return,
                _ => {}
            }
            if in_switch {
                let save = self.p;
                let w = self.word().to_ascii_lowercase();
                self.p = save;
                if w == "case" || w == "default" { return; }
            }
            let before = self.p;
            self.stmt(sc, out);
            if self.p == before { self.p += 1; }
        }
    }

    fn stmt(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        self.skip();
        let name = self.word();
        let lname = name.to_ascii_lowercase();
        match lname.as_str() {
            "" => { self.p += 1; }
            "if" => {
                let mut taken: Vec<String> = vec![];
                let mut cond = self.until(b"{");
                loop {
                    let mut inner = sc.clone();
                    inner.conds.extend(taken.iter().map(|c| format!("!({c})")));
                    inner.conds.push(cond.clone());
                    self.block_or_stmt(&inner, out);
                    taken.push(cond);
                    self.skip();
                    let save = self.p;
                    if !self.word().eq_ignore_ascii_case("else") { self.p = save; return; }
                    self.skip();
                    let save = self.p;
                    if self.word().eq_ignore_ascii_case("if") { cond = self.until(b"{"); continue; }
                    self.p = save;
                    let mut inner = sc.clone();
                    inner.conds.extend(taken.iter().map(|c| format!("!({c})")));
                    self.block_or_stmt(&inner, out);
                    return;
                }
            }
            "switch" => {
                let subject = self.until(b"{");
                self.skip();
                if self.peek() != Some(b'{') { return; }
                self.p += 1;
                let mut taken: Vec<String> = vec![];
                loop {
                    self.skip();
                    let w = self.word().to_ascii_lowercase();
                    let mut inner = sc.clone();
                    inner.conds.extend(taken.iter().map(|c| format!("!({c})")));
                    match w.as_str() {
                        "case" => {
                            let vals = self.until(b":");
                            self.p += 1;
                            let cond = vals.split(',').map(|v| format!("({subject}) = ({})", v.trim())).collect::<Vec<_>>().join(" || ");
                            inner.conds.push(cond.clone());
                            taken.push(cond);
                            self.stmts(&inner, out, true);
                        }
                        "default" => {
                            self.until(b":");
                            self.p += 1;
                            self.stmts(&inner, out, true);
                        }
                        _ => break,
                    }
                }
                self.skip();
                if self.peek() == Some(b'}') { self.p += 1; }
            }
            "ignorehitpause" => {
                let mut inner = sc.clone();
                inner.ignore_hitpause = true;
                self.block_or_stmt(&inner, out);
            }
            "persistent" => {
                self.skip();
                let mut inner = sc.clone();
                if self.peek() == Some(b'(') {
                    self.p += 1;
                    let n = self.until(b")");
                    self.p += 1;
                    inner.persistent = n.trim().parse().ok();
                }
                self.block_or_stmt(&inner, out);
            }
            "let" | "call" | "break" | "continue" => {
                self.unsupported.push(lname.clone());
                self.until(b";\n}");
            }
            "for" | "while" => {
                self.unsupported.push(lname.clone());
                self.until(b"{");
                self.braced();
            }
            _ => {
                self.skip();
                if self.peek() != Some(b'{') { self.until(b";\n}"); return; }
                let body = self.braced();
                if let Some(c) = controller(&name, &body, sc) { out.push(c); }
            }
        }
    }
}

fn split_top(s: &str, sep: u8) -> Vec<String> {
    let b = s.as_bytes();
    let (mut out, mut st, mut depth, mut q) = (vec![], 0, 0i32, false);
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'"' => q = !q,
            b'(' | b'[' if !q => depth += 1,
            b')' | b']' if !q => depth -= 1,
            _ if c == sep && depth == 0 && !q => { out.push(s[st..i].to_string()); st = i + 1; }
            _ => {}
        }
    }
    out.push(s[st..].to_string());
    out.into_iter().map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

fn key_values(body: &str) -> Vec<(String, String)> {
    let body: String = body.lines().map(strip_comment).collect::<Vec<_>>().join("\n");
    split_top(&body, b';').into_iter().flat_map(|part| split_top(&part, b'\n')).filter_map(|kv| {
        let (k, v) = kv.split_once(':')?;
        Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
    }).collect()
}

fn strip_comment(l: &str) -> &str {
    let mut q = false;
    for (i, c) in l.char_indices() {
        match c { '"' => q = !q, '#' if !q => return &l[..i], _ => {} }
    }
    l
}

fn controller(name: &str, body: &str, sc: &Scope) -> Option<crate::chardef::Controller> {
    let mut entries = vec![("type".to_string(), name.to_string())];
    entries.extend(key_values(body));
    let sec = Section { name: format!("State {name}"), lname: format!("state {}", name.to_ascii_lowercase()), entries, lines: vec![] };
    let mut c = build_controller(&sec)?;
    c.trigger_all = sc.conds.iter().map(|x| crate::chardef::safe_expr(x)).chain(c.trigger_all).collect();
    if c.triggers.is_empty() { c.triggers = vec![vec![Expr::num(1.0)]]; }
    if let Some(p) = sc.persistent { c.persistent = p; }
    if sc.ignore_hitpause { c.ignore_hitpause = true; }
    Some(c)
}

pub struct ZssReport { pub states: usize, pub unsupported: Vec<String> }

pub fn parse_zss(text: &str, into: &mut HashMap<i32, StateDef>, override_existing: bool) -> ZssReport {
    let mut r = Reader { s: text.as_bytes(), p: 0, unsupported: vec![] };
    let mut states = 0;
    loop {
        r.skip();
        match r.peek() {
            None => break,
            Some(b'[') => {}
            Some(_) => { r.p += 1; continue; }
        }
        r.p += 1;
        let header = r.until(b"]");
        r.p += 1;
        let parts = split_top(&header, b';');
        let head = parts.first().cloned().unwrap_or_default();
        let lhead = head.to_ascii_lowercase();
        let Some(rest) = lhead.strip_prefix("statedef") else {
            r.unsupported.push(lhead.split_whitespace().next().unwrap_or("").to_string());
            let mut sink = vec![];
            r.stmts(&Scope::default(), &mut sink, false);
            continue;
        };
        let id: i32 = rest.trim().parse().unwrap_or(i32::MIN);
        let mut ctrls = vec![];
        r.stmts(&Scope::default(), &mut ctrls, false);
        if id == i32::MIN || (into.contains_key(&id) && !override_existing && id >= 0) { continue; }
        states += 1;
        let params: Vec<(String, Param)> = parts.iter().skip(1).filter_map(|kv| kv.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), Param::new(v.trim()))).collect();
        if id < 0 && into.contains_key(&id) {
            into.get_mut(&id).unwrap().controllers.extend(ctrls);
        } else {
            into.insert(id, StateDef { id, params, controllers: ctrls });
        }
    }
    r.unsupported.sort();
    r.unsupported.dedup();
    ZssReport { states, unsupported: r.unsupported }
}

#[cfg(test)]
mod tests;

pub fn parse_state_file(name: &str, text: &str, into: &mut HashMap<i32, StateDef>, override_existing: bool) {
    if name.trim().trim_matches('"').to_ascii_lowercase().ends_with(".zss") {
        let r = parse_zss(text, into, override_existing);
        if !r.unsupported.is_empty() { eprintln!("zss {name}: unsupported {}", r.unsupported.join(", ")); }
    } else {
        crate::cns::parse_states(text, into, override_existing);
    }
}
