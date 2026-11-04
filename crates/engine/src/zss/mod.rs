use crate::chardef::{Controller, Param, StateDef};
use crate::expr::{is_float, parse, Expr};
use crate::cns::build_controller;
use crate::fighter::LOCAL_SLOTS;
use std::collections::HashMap;
use ume_formats::ini::Section;

pub const LOOP_ITERATION_CAP: u32 = 2500;
const COMPONENT_TRIGGERS: [&str; 13] = ["vel", "pos", "p2dist", "p2bodydist", "p1dist", "rootdist", "parentdist", "screenpos", "camerapos", "hitvel", "camerazoom", "scale", "offset"];
const CALL_DEPTH_LIMIT: usize = 16;

#[derive(Clone, Default)]
struct Scope { conds: Vec<String>, persistent: Option<i32>, ignore_hitpause: bool }

#[derive(Clone, Copy)]
struct Local { slot: usize, float: bool }

struct NameScope { names: HashMap<String, Local>, barrier: bool, first_slot: usize }

#[derive(Default)]
struct LoopCtx { breaks: Vec<usize>, continues: Vec<usize> }

struct Frame { name: String, ret_slot: usize, ret_float: Option<bool>, returns: Vec<usize> }

#[derive(Clone)]
struct Function { params: Vec<String>, body: usize }

struct Reader<'a> {
    s: &'a [u8],
    p: usize,
    errors: Vec<String>,
    funcs: HashMap<String, Function>,
    scopes: Vec<NameScope>,
    next_slot: usize,
    loops: Vec<LoopCtx>,
    frames: Vec<Frame>,
}

fn split_assign(text: &str) -> Option<(bool, String, &'static str, String)> {
    let mut t = text.trim();
    let mut declared = false;
    if t.len() > 3 && t[..3].eq_ignore_ascii_case("let") && t[3..].starts_with(|c: char| c.is_whitespace() || c == '$') { declared = true; t = t[3..].trim_start(); }
    let t = t.strip_prefix('$').unwrap_or(t);
    let end = t.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(t.len());
    if end == 0 || t.starts_with(|c: char| c.is_ascii_digit()) { return None; }
    let name = t[..end].to_ascii_lowercase();
    let rest = t[end..].trim_start();
    for op in [":=", "+=", "-=", "*=", "/=", "%=", "++", "--"] {
        if let Some(r) = rest.strip_prefix(op) { return Some((declared, name, op, r.trim().to_string())); }
    }
    let r = rest.strip_prefix('=')?;
    if r.starts_with('=') { return None; }
    Some((declared, name, "=", r.trim().to_string()))
}

fn call_parts(text: &str) -> Option<(String, Vec<String>)> {
    let t = text.trim();
    let t = if t.len() > 4 && t[..4].eq_ignore_ascii_case("call") && t[4..].starts_with(|c: char| c.is_whitespace()) { t[4..].trim_start() } else { t };
    let open = t.find('(')?;
    let name = t[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') { return None; }
    let close = t.rfind(')')?;
    if close < open || !t[close + 1..].trim().is_empty() { return None; }
    Some((name.to_ascii_lowercase(), split_top(&t[open + 1..close], b',')))
}

fn starts_with_call(text: &str) -> bool {
    let t = text.trim_start();
    t.len() > 4 && t[..4].eq_ignore_ascii_case("call") && t[4..].starts_with(|c: char| c.is_whitespace())
}

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
            self.push_scope(false);
            self.stmts(sc, out, false);
            self.pop_scope();
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
        if self.peek() == Some(b'$') {
            let text = self.until(b";\n}");
            self.assign_text(&text, sc, out);
            return;
        }
        let name = self.word();
        let lname = name.to_ascii_lowercase();
        match lname.as_str() {
            "" => { self.p += 1; }
            "if" => {
                let mut taken: Vec<String> = vec![];
                let mut cond = self.condition(b"{");
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
                    if self.word().eq_ignore_ascii_case("if") { cond = self.condition(b"{"); continue; }
                    self.p = save;
                    let mut inner = sc.clone();
                    inner.conds.extend(taken.iter().map(|c| format!("!({c})")));
                    self.block_or_stmt(&inner, out);
                    return;
                }
            }
            "switch" => {
                let subject = self.condition(b"{");
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
                            let cond = vals.split(',').map(|v| { let v = self.rw(v.trim()); self.check_expr(&v); format!("({subject}) = ({v})") }).collect::<Vec<_>>().join(" || ");
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
            "let" => {
                let rest = self.until(b";\n}");
                self.let_stmt(&format!("let {rest}"), sc, out);
            }
            "call" => {
                let rest = self.until(b";\n}");
                self.call_stmt(&rest, sc, out, None);
            }
            "break" | "continue" => {
                self.until(b";\n}");
                if self.loops.is_empty() { self.fail(format!("`{lname}` outside of a loop")); return; }
                let at = self.emit_jump(sc, out, None);
                let ctx = self.loops.last_mut().unwrap();
                if lname == "break" { ctx.breaks.push(at); } else { ctx.continues.push(at); }
            }
            "return" => {
                let text = self.until(b";\n}");
                self.return_stmt(&text, sc, out);
            }
            "while" => {
                let cond = self.condition(b"{");
                self.loop_stmt(sc, out, cond, None);
            }
            "for" => {
                let header = self.until(b"{");
                let parts: Vec<&str> = header.split(';').collect();
                if parts.len() != 3 { self.fail("`for` needs `init; condition; step`".into()); self.braced(); return; }
                self.push_scope(false);
                if !parts[0].trim().is_empty() { self.assign_text(parts[0], sc, out); }
                let cond = if parts[1].trim().is_empty() { "1".to_string() } else { let c = self.rw(parts[1]); self.check_expr(&c); c };
                self.loop_stmt(sc, out, cond, Some(parts[2].trim().to_string()));
                self.pop_scope();
            }
            _ => {
                self.skip();
                if self.peek() == Some(b'{') {
                    let body = self.braced();
                    if let Some(c) = self.controller(&name, &body, sc) { out.push(c); }
                    return;
                }
                let rest = self.until(b";\n}");
                let text = format!("{name}{rest}");
                if self.funcs.contains_key(&lname) && rest.trim_start().starts_with('(') { self.call_stmt(&text, sc, out, None); }
                else if self.lookup(&lname).is_some() { self.assign_text(&text, sc, out); }
                else if matches!(lname.as_str(), "var" | "fvar" | "sysvar" | "sysfvar") && rest.trim_start().starts_with('(') {
                    let e = self.rw(&text);
                    self.check_expr(&e);
                    if let Some(c) = self.build("null", vec![("trigger1".into(), e)], &[], sc) { out.push(c); }
                } else { self.fail(format!("unsupported statement `{text}`")); }
            }
        }
    }

    fn skip_section(&mut self) {
        while self.p < self.s.len() && !(self.s[self.p] == b'[' && (self.p == 0 || self.s[..self.p].iter().rev().take_while(|&&c| c != b'\n').all(|c| c.is_ascii_whitespace()))) { self.p += 1; }
    }

    fn line(&self) -> usize { self.s[..self.p.min(self.s.len())].iter().filter(|&&c| c == b'\n').count() + 1 }

    fn fail(&mut self, msg: String) {
        let line = self.line();
        self.errors.push(format!("line {line}: {msg}"));
    }

    fn check_expr(&mut self, text: &str) {
        if let Err(e) = parse(text) { self.fail(format!("invalid expression `{text}`: {e}")); }
    }

    fn condition(&mut self, stops: &[u8]) -> String {
        let raw = self.until(stops);
        let c = self.rw(&raw);
        self.check_expr(&c);
        c
    }

    fn push_scope(&mut self, barrier: bool) { self.scopes.push(NameScope { names: HashMap::new(), barrier, first_slot: self.next_slot }); }

    fn pop_scope(&mut self) {
        if let Some(sc) = self.scopes.pop() { self.next_slot = sc.first_slot; }
    }

    fn lookup(&self, name: &str) -> Option<Local> {
        for sc in self.scopes.iter().rev() {
            if let Some(l) = sc.names.get(name) { return Some(*l); }
            if sc.barrier { break; }
        }
        None
    }

    fn alloc(&mut self) -> Option<usize> {
        if self.next_slot >= LOCAL_SLOTS { self.fail(format!("more than {LOCAL_SLOTS} live local variables")); return None; }
        self.next_slot += 1;
        Some(self.next_slot - 1)
    }

    fn declare(&mut self, name: &str, float: bool) -> Option<Local> {
        let slot = self.alloc()?;
        let l = Local { slot, float };
        self.scopes.last_mut()?.names.insert(name.to_string(), l);
        Some(l)
    }

    fn rw(&mut self, text: &str) -> String {
        let b = text.as_bytes();
        let mut out = String::with_capacity(text.len() + 8);
        let (mut i, mut prev) = (0, String::new());
        while i < b.len() {
            let c = b[i];
            if c == b'"' {
                let st = i;
                i += 1;
                while i < b.len() && b[i] != b'"' { i += 1; }
                i = (i + 1).min(b.len());
                out.push_str(&text[st..i]);
                prev.clear();
                continue;
            }
            if c.is_ascii_digit() {
                let st = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') { i += 1; }
                out.push_str(&text[st..i]);
                prev.clear();
                continue;
            }
            if c == b'$' || c.is_ascii_alphabetic() || c == b'_' {
                let dollar = c == b'$';
                if dollar { i += 1; }
                let st = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') { i += 1; }
                let word = &text[st..i];
                let lw = word.to_ascii_lowercase();
                let next_paren = text[i..].trim_start().starts_with('(');
                let component = !dollar && lw.len() == 1 && matches!(lw.as_str(), "x" | "y" | "z") && COMPONENT_TRIGGERS.contains(&prev.as_str());
                let local = if component || (next_paren && !dollar) { None } else { self.lookup(&lw) };
                if let Some(l) = local {
                    out.push_str(&format!("{}({})", if l.float { "flocal" } else { "local" }, l.slot));
                    prev.clear();
                } else if dollar {
                    self.fail(format!("unknown local variable `${word}`"));
                    out.push('0');
                } else {
                    if next_paren && self.funcs.contains_key(&lw) { self.fail(format!("function `{word}` used inside an expression; use `let r = call {word}(...)`")); }
                    out.push_str(word);
                    prev = lw;
                }
                continue;
            }
            out.push(c as char);
            if !c.is_ascii_whitespace() { prev.clear(); }
            i += 1;
        }
        out
    }

    fn build(&mut self, kind: &str, entries: Vec<(String, String)>, extra: &[String], sc: &Scope) -> Option<Controller> {
        let mut all = vec![("type".to_string(), kind.to_string())];
        all.extend(entries);
        let sec = Section { name: format!("State {kind}"), lname: format!("state {}", kind.to_ascii_lowercase()), entries: all, lines: vec![] };
        let mut c = build_controller(&sec)?;
        c.trigger_all = sc.conds.iter().chain(extra.iter()).map(|x| crate::chardef::safe_expr(x)).chain(c.trigger_all).collect();
        if c.triggers.is_empty() { c.triggers = vec![vec![Expr::num(1.0)]]; }
        if let Some(p) = sc.persistent { c.persistent = p; }
        if sc.ignore_hitpause { c.ignore_hitpause = true; }
        Some(c)
    }

    fn controller(&mut self, name: &str, body: &str, sc: &Scope) -> Option<Controller> {
        let entries = key_values(body).into_iter().map(|(k, v)| (k, self.rw(&v))).collect();
        self.build(name, entries, &[], sc)
    }

    fn emit_set(&mut self, local: Local, value: &str, sc: &Scope, out: &mut Vec<Controller>) {
        let entries = vec![("slot".to_string(), local.slot.to_string()), ("value".to_string(), value.to_string()), ("float".to_string(), (local.float as i32).to_string())];
        if let Some(c) = self.build("__set", entries, &[], sc) { out.push(c); }
    }

    fn emit_jump(&mut self, sc: &Scope, out: &mut Vec<Controller>, extra: Option<String>) -> usize {
        let at = out.len();
        let mut flat = sc.clone();
        flat.persistent = None;
        let extra: Vec<String> = extra.into_iter().collect();
        if let Some(c) = self.build("__jump", vec![("to".into(), "0".into()), ("exit".into(), "1".into())], &extra, &flat) { out.push(c); }
        at
    }

    fn patch_jump(out: &mut [Controller], at: usize, target: usize, exit: Option<usize>) {
        let Some(c) = out.get_mut(at) else { return };
        let rel = |t: usize| (t as i64 - at as i64).to_string();
        for (k, p) in c.params.iter_mut() {
            if k == "to" { *p = Param::new(&rel(target)); }
            if k == "exit" { if let Some(e) = exit { *p = Param::new(&rel(e)); } }
        }
    }

    fn assign_text(&mut self, text: &str, sc: &Scope, out: &mut Vec<Controller>) {
        let Some((declared, name, op, rhs)) = split_assign(text) else { self.fail(format!("cannot parse statement `{}`", text.trim())); return };
        if declared || (op == "=" || op == ":=") && self.lookup(&name).is_none() && !starts_with_call(&rhs) { self.let_stmt(text, sc, out); return; }
        let Some(local) = self.lookup(&name) else { self.fail(format!("unknown local variable `{name}`")); return };
        let reference = format!("{}({})", if local.float { "flocal" } else { "local" }, local.slot);
        let value = if starts_with_call(&rhs) {
            let Some((slot, float)) = self.call_stmt(&rhs, sc, out, Some(())) else { return };
            format!("{}({slot})", if float { "flocal" } else { "local" })
        } else {
            let r = self.rw(&rhs);
            if op != "++" && op != "--" { self.check_expr(&r); }
            match op {
                "++" => format!("{reference} + 1"),
                "--" => format!("{reference} - 1"),
                "=" | ":=" => r,
                _ => format!("{reference} {} ({r})", &op[..1]),
            }
        };
        self.emit_set(local, &value, sc, out);
    }

    fn let_stmt(&mut self, text: &str, sc: &Scope, out: &mut Vec<Controller>) {
        let Some((_, name, op, rhs)) = split_assign(text) else { self.fail(format!("cannot parse `{}`", text.trim())); return };
        if op != "=" && op != ":=" { self.fail(format!("`let {name}` needs `=`")); return; }
        if starts_with_call(&rhs) {
            let Some((slot, float)) = self.call_stmt(&rhs, sc, out, Some(())) else { return };
            if let Some(top) = self.scopes.last_mut() { top.names.insert(name, Local { slot, float }); }
            return;
        }
        let value = self.rw(&rhs);
        self.check_expr(&value);
        let float = parse(&value).map(|n| is_float(&n)).unwrap_or(false);
        let Some(local) = self.declare(&name, float) else { return };
        self.emit_set(local, &value, sc, out);
    }

    fn loop_stmt(&mut self, sc: &Scope, out: &mut Vec<Controller>, cond: String, step: Option<String>) {
        let start = out.len();
        let exit_jump = self.emit_jump(sc, out, Some(format!("!({cond})")));
        self.loops.push(LoopCtx::default());
        self.block_or_stmt(sc, out);
        let ctx = self.loops.pop().unwrap_or_default();
        let step_at = out.len();
        if let Some(step) = step { if !step.is_empty() { self.assign_text(&step, sc, out); } }
        let back = out.len();
        self.emit_jump(sc, out, None);
        let end = out.len();
        Self::patch_jump(out, exit_jump, end, None);
        Self::patch_jump(out, back, start, Some(end));
        for b in ctx.breaks { Self::patch_jump(out, b, end, None); }
        for c in ctx.continues { Self::patch_jump(out, c, step_at, None); }
    }

    fn return_stmt(&mut self, text: &str, sc: &Scope, out: &mut Vec<Controller>) {
        if self.frames.is_empty() { self.fail("`return` outside of a function".into()); return; }
        if !text.trim().is_empty() {
            let value = self.rw(text);
            self.check_expr(&value);
            let float = parse(&value).map(|n| is_float(&n)).unwrap_or(false);
            let frame = self.frames.last_mut().unwrap();
            let kind = *frame.ret_float.get_or_insert(float);
            let slot = frame.ret_slot;
            self.emit_set(Local { slot, float: kind }, &value, sc, out);
        }
        let at = self.emit_jump(sc, out, None);
        self.frames.last_mut().unwrap().returns.push(at);
    }

    fn call_stmt(&mut self, text: &str, sc: &Scope, out: &mut Vec<Controller>, want: Option<()>) -> Option<(usize, bool)> {
        let Some((name, args)) = call_parts(text) else { self.fail(format!("cannot parse call `{}`", text.trim())); return None };
        let Some(func) = self.funcs.get(&name).cloned() else { self.fail(format!("call to undefined function `{name}`")); return None };
        if self.frames.iter().any(|f| f.name == name) { self.fail(format!("recursive call to `{name}`")); return None; }
        if self.frames.len() >= CALL_DEPTH_LIMIT { self.fail(format!("calls nested deeper than {CALL_DEPTH_LIMIT}")); return None; }
        if args.len() != func.params.len() { self.fail(format!("`{name}` takes {} arguments, got {}", func.params.len(), args.len())); return None; }
        let values: Vec<String> = args.iter().map(|a| { let v = self.rw(a); self.check_expr(&v); v }).collect();
        let base_slot = self.next_slot;
        let ret_slot = self.alloc()?;
        self.push_scope(true);
        for (param, value) in func.params.iter().zip(&values) {
            let float = parse(value).map(|n| is_float(&n)).unwrap_or(false);
            let local = self.declare(param, float)?;
            self.emit_set(local, value, sc, out);
        }
        self.frames.push(Frame { name: name.clone(), ret_slot, ret_float: None, returns: vec![] });
        let saved_p = self.p;
        let saved_loops = std::mem::take(&mut self.loops);
        self.p = func.body;
        self.stmts(sc, out, false);
        self.p = saved_p;
        self.loops = saved_loops;
        let frame = self.frames.pop()?;
        let end = out.len();
        for at in frame.returns { Self::patch_jump(out, at, end, None); }
        self.pop_scope();
        if want.is_none() { self.next_slot = base_slot; }
        Some((ret_slot, frame.ret_float.unwrap_or(false)))
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

pub struct ZssReport { pub states: usize, pub errors: Vec<String> }

fn section_starts(s: &[u8]) -> Vec<usize> {
    let (mut out, mut depth, mut i, mut line_start) = (vec![], 0i32, 0, true);
    while i < s.len() {
        let c = s[i];
        match c {
            b'"' => { i += 1; while i < s.len() && s[i] != b'"' && s[i] != b'\n' { i += 1; } }
            b'#' => { while i < s.len() && s[i] != b'\n' { i += 1; } continue; }
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'[' if depth <= 0 && line_start => out.push(i),
            _ => {}
        }
        if c == b'\n' { line_start = true; } else if !c.is_ascii_whitespace() { line_start = false; }
        i += 1;
    }
    out
}

fn function_header(head: &str) -> Option<(String, Vec<String>)> {
    let head = head.trim();
    if !head.get(..8)?.eq_ignore_ascii_case("function") { return None; }
    let rest = head.get(8..)?;
    if !rest.starts_with(|c: char| c.is_whitespace()) { return None; }
    let rest = rest.trim();
    let (name, params) = match rest.find('(') {
        Some(open) => {
            let close = rest.rfind(')')?;
            (rest[..open].trim(), split_top(&rest[open + 1..close], b',').into_iter().map(|p| p.trim_start_matches('$').to_ascii_lowercase()).collect())
        }
        None => (rest, vec![]),
    };
    Some((name.to_ascii_lowercase(), params))
}

pub fn parse_zss(text: &str, into: &mut HashMap<i32, StateDef>, override_existing: bool) -> ZssReport {
    let mut r = Reader { s: text.as_bytes(), p: 0, errors: vec![], funcs: HashMap::new(), scopes: vec![], next_slot: 0, loops: vec![], frames: vec![] };
    for start in section_starts(r.s) {
        r.p = start + 1;
        let header = r.until(b"]");
        let head = split_top(&header, b';').first().cloned().unwrap_or_default();
        if head.get(..8).is_some_and(|h| h.eq_ignore_ascii_case("function")) {
            match function_header(&head) {
                Some((name, params)) => { r.funcs.insert(name, Function { params, body: (r.p + 1).min(r.s.len()) }); }
                None => r.fail(format!("malformed function header `{head}`")),
            }
        }
    }
    r.p = 0;
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
            if !lhead.starts_with("function") { r.fail(format!("unsupported section `[{head}]`")); }
            r.skip_section();
            continue;
        };
        let id: i32 = rest.trim().parse().unwrap_or(i32::MIN);
        r.scopes.clear();
        r.push_scope(true);
        r.next_slot = 0;
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
    ZssReport { states, errors: r.errors }
}

#[cfg(test)]
mod tests;

pub fn parse_state_file(name: &str, text: &str, into: &mut HashMap<i32, StateDef>, override_existing: bool) -> Result<(), String> {
    if name.trim().trim_matches('"').to_ascii_lowercase().ends_with(".zss") {
        let r = parse_zss(text, into, override_existing);
        if !r.errors.is_empty() { return Err(format!("zss {name}: {}", r.errors.join("; "))); }
    } else {
        crate::cns::parse_states(text, into, override_existing);
    }
    Ok(())
}
