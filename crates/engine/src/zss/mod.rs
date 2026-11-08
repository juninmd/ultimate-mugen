use crate::chardef::{Param, StateDef};
use crate::cns::build_controller;
use crate::expr::Expr;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use ume_formats::ini::Section;

const LOCALS_TOP: i32 = 59;
const LOCALS_MIN: i32 = 40;
const LOOP_CAP: usize = 32;

#[derive(Clone, Default)]
struct Scope {
    conds: Vec<String>,
    persistent: Option<i32>,
    ignore_hitpause: bool,
}

#[derive(Clone)]
struct FuncDef {
    params: Vec<String>,
    body: String,
}

struct LoopCtx {
    break_slot: usize,
    cont_slot: usize,
}

struct Reader<'a> {
    s: &'a [u8],
    p: usize,
    unsupported: Rc<RefCell<Vec<String>>>,
    errors: Rc<RefCell<Vec<String>>>,
    locals: Rc<RefCell<HashMap<String, usize>>>,
    next_local: Rc<Cell<i32>>,
    funcs: Rc<HashMap<String, FuncDef>>,
    loops: Vec<LoopCtx>,
    extra: Vec<String>,
    depth: u8,
}

impl<'a> Reader<'a> {
    fn child<'b>(&self, s: &'b [u8]) -> Reader<'b> {
        Reader {
            s,
            p: 0,
            unsupported: self.unsupported.clone(),
            errors: self.errors.clone(),
            locals: self.locals.clone(),
            next_local: self.next_local.clone(),
            funcs: self.funcs.clone(),
            loops: vec![],
            extra: vec![],
            depth: self.depth.saturating_add(1),
        }
    }

    fn error(&self, msg: String) {
        self.errors.borrow_mut().push(msg);
    }

    fn skip(&mut self) {
        while self.p < self.s.len() {
            match self.s[self.p] {
                b'#' => {
                    while self.p < self.s.len() && self.s[self.p] != b'\n' {
                        self.p += 1;
                    }
                }
                c if c.is_ascii_whitespace() || c == b';' => self.p += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.p).copied()
    }

    fn word(&mut self) -> String {
        let st = self.p;
        while self.p < self.s.len()
            && (self.s[self.p].is_ascii_alphanumeric()
                || self.s[self.p] == b'_'
                || self.s[self.p] == b'.')
        {
            self.p += 1;
        }
        String::from_utf8_lossy(&self.s[st..self.p]).into_owned()
    }

    fn until(&mut self, stops: &[u8]) -> String {
        let st = self.p;
        let mut depth = 0i32;
        while self.p < self.s.len() {
            let c = self.s[self.p];
            match c {
                b'"' => {
                    self.p += 1;
                    while self.p < self.s.len() && self.s[self.p] != b'"' {
                        self.p += 1;
                    }
                }
                b'#' if depth == 0 => break,
                b'(' | b'[' => depth += 1,
                b')' | b']' if depth > 0 => depth -= 1,
                _ if depth == 0 && stops.contains(&c) => break,
                _ => {}
            }
            self.p += 1;
        }
        let out = String::from_utf8_lossy(&self.s[st..self.p])
            .trim()
            .to_string();
        if self.p < self.s.len() && self.s[self.p] == b'#' {
            while self.p < self.s.len() && self.s[self.p] != b'\n' {
                self.p += 1;
            }
        }
        out
    }

    fn braced(&mut self) -> String {
        self.skip();
        if self.peek() != Some(b'{') {
            return String::new();
        }
        self.p += 1;
        let st = self.p;
        let mut depth = 1;
        while self.p < self.s.len() {
            match self.s[self.p] {
                b'"' => {
                    self.p += 1;
                    while self.p < self.s.len() && self.s[self.p] != b'"' {
                        self.p += 1;
                    }
                }
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            self.p += 1;
        }
        let out = String::from_utf8_lossy(&self.s[st..self.p.min(self.s.len())]).into_owned();
        self.p += 1;
        out
    }

    fn declare(&self, name: &str) -> usize {
        let key = name.trim().to_ascii_lowercase();
        if let Some(&slot) = self.locals.borrow().get(&key) {
            return slot;
        }
        let slot = self.next_local.get();
        if slot < LOCALS_MIN {
            self.error(format!(
                "zss: too many local variables (limit {})",
                LOCALS_TOP - LOCALS_MIN + 1
            ));
            return LOCALS_MIN as usize;
        }
        self.next_local.set(slot - 1);
        self.locals.borrow_mut().insert(key, slot as usize);
        slot as usize
    }

    fn alloc_temp(&self) -> usize {
        let slot = self.next_local.get();
        if slot >= LOCALS_MIN {
            self.next_local.set(slot - 1);
            slot as usize
        } else {
            LOCALS_MIN as usize
        }
    }

    fn rewrite(&self, s: &str) -> String {
        let b = s.as_bytes();
        let mut out = String::with_capacity(s.len());
        let (mut i, mut quote) = (0usize, false);
        while i < b.len() {
            let c = b[i];
            if c == b'"' {
                quote = !quote;
                out.push('"');
                i += 1;
                continue;
            }
            if !quote && (c.is_ascii_alphabetic() || c == b'_') {
                let st = i;
                while i < b.len()
                    && ((b[i] as char).is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.')
                {
                    i += 1;
                }
                let word = &s[st..i];
                if let Some(&slot) = self.locals.borrow().get(&word.to_ascii_lowercase()) {
                    out.push_str(&format!("var({slot})"));
                } else {
                    out.push_str(word);
                }
                continue;
            }
            out.push(c as char);
            i += 1;
        }
        out
    }

    fn rewritten_scope(&self, sc: &Scope) -> Scope {
        let mut conds: Vec<String> = sc.conds.iter().map(|c| self.rewrite(c)).collect();
        conds.extend(self.extra.iter().map(|c| self.rewrite(c)));
        Scope {
            conds,
            persistent: sc.persistent,
            ignore_hitpause: sc.ignore_hitpause,
        }
    }

    fn emit(&self, name: &str, body: &str, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        let body = self.rewrite(body);
        if let Some(c) = controller(name, &body, &self.rewritten_scope(sc)) {
            out.push(c);
        }
    }

    fn var_set(
        &self,
        slot: usize,
        expr: &str,
        sc: &Scope,
        out: &mut Vec<crate::chardef::Controller>,
    ) {
        self.emit("VarSet", &format!("v: {slot}; value: {}", expr), sc, out);
    }

    fn var_add(
        &self,
        slot: usize,
        expr: &str,
        sc: &Scope,
        out: &mut Vec<crate::chardef::Controller>,
    ) {
        self.emit("VarAdd", &format!("v: {slot}; value: {}", expr), sc, out);
    }

    fn assignment(&self, text: &str, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        let t = text.trim();
        let t = t.strip_prefix("let ").unwrap_or(t).trim();
        if let Some(name) = t.strip_suffix("++") {
            self.var_add(self.declare(name.trim()), "1", sc, out);
            return;
        }
        if let Some(name) = t.strip_suffix("--") {
            self.var_add(self.declare(name.trim()), "-1", sc, out);
            return;
        }
        if let Some((name, expr)) = t.split_once("+=") {
            self.var_add(self.declare(name.trim()), expr.trim(), sc, out);
            return;
        }
        if let Some((name, expr)) = t.split_once("-=") {
            self.var_add(
                self.declare(name.trim()),
                &format!("-({})", expr.trim()),
                sc,
                out,
            );
            return;
        }
        let Some((name, expr)) = t.split_once('=') else {
            self.error(format!("zss: unsupported statement '{t}'"));
            return;
        };
        self.var_set(self.declare(name.trim()), expr.trim(), sc, out);
    }

    fn assign_next(&self, lname: &str) -> bool {
        if !self.locals.borrow().contains_key(lname) {
            return false;
        }
        let mut p = self.p;
        while p < self.s.len() && (self.s[p] as char).is_ascii_whitespace() {
            p += 1;
        }
        matches!(self.s.get(p), Some(b'=') | Some(b'+') | Some(b'-'))
    }

    fn block_or_stmt(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        self.skip();
        if self.peek() == Some(b'{') {
            self.p += 1;
            self.stmts(sc, out, false);
            self.skip();
            if self.peek() == Some(b'}') {
                self.p += 1;
            }
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
                if w == "case" || w == "default" {
                    return;
                }
            }
            let before = self.p;
            self.stmt(sc, out);
            if self.p == before {
                self.p += 1;
            }
        }
    }

    fn stmt(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        self.skip();
        let name = self.word();
        let lname = name.to_ascii_lowercase();
        if self.assign_next(&lname) {
            let text = self.until(b";}");
            if self.peek() == Some(b';') {
                self.p += 1;
            }
            self.assignment(&text, sc, out);
            return;
        }
        match lname.as_str() {
            "" => {
                self.p += 1;
            }
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
                    if !self.word().eq_ignore_ascii_case("else") {
                        self.p = save;
                        return;
                    }
                    self.skip();
                    let save = self.p;
                    if self.word().eq_ignore_ascii_case("if") {
                        cond = self.until(b"{");
                        continue;
                    }
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
                if self.peek() != Some(b'{') {
                    return;
                }
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
                            let cond = vals
                                .split(',')
                                .map(|v| format!("({subject}) = ({})", v.trim()))
                                .collect::<Vec<_>>()
                                .join(" || ");
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
                if self.peek() == Some(b'}') {
                    self.p += 1;
                }
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
                let text = self.until(b";}");
                if self.peek() == Some(b';') {
                    self.p += 1;
                }
                self.assignment(&text, sc, out);
            }
            "call" => {
                self.skip();
                let text = self.until(b";}");
                if self.peek() == Some(b';') {
                    self.p += 1;
                }
                self.expand_call(&text, sc, out);
            }
            "break" | "continue" => {
                self.until(b";}");
                if self.peek() == Some(b';') {
                    self.p += 1;
                }
                let Some(l) = self.loops.last() else {
                    self.error(format!("zss: '{lname}' outside a loop"));
                    return;
                };
                if lname == "break" {
                    self.var_set(l.break_slot, "1", sc, out);
                } else {
                    self.var_set(l.cont_slot, "1", sc, out);
                    self.extra.push(format!("var({}) = 0", l.cont_slot));
                }
            }
            "for" => self.for_loop(sc, out),
            "while" => self.while_loop(sc, out),
            _ => {
                if self.funcs.contains_key(&lname) {
                    self.skip();
                    if self.peek() == Some(b'(') {
                        let mut depth = 0i32;
                        let st = self.p;
                        while self.p < self.s.len() {
                            match self.s[self.p] {
                                b'(' => depth += 1,
                                b')' => {
                                    depth -= 1;
                                    if depth == 0 {
                                        self.p += 1;
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            self.p += 1;
                        }
                        let args =
                            String::from_utf8_lossy(&self.s[st + 1..self.p.saturating_sub(1)])
                                .into_owned();
                        self.until(b";}");
                        if self.peek() == Some(b';') {
                            self.p += 1;
                        }
                        self.expand_call(&format!("{name}({args})"), sc, out);
                        return;
                    }
                }
                self.skip();
                if self.peek() != Some(b'{') {
                    self.until(b";\n}");
                    return;
                }
                let body = self.braced();
                self.emit(&name, &body, sc, out);
            }
        }
    }

    fn for_loop(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        let header = self.until(b"{");
        let parts = split_top(&header, b';');
        if parts.len() != 3 {
            self.error(format!("zss: malformed for loop '{header}'"));
            self.skip_braced();
            return;
        }
        self.assignment(&parts[0], sc, out);
        self.loop_body(sc, out, &self.rewrite(&parts[1]), Some(parts[2].clone()));
    }

    fn while_loop(&mut self, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        let cond = self.until(b"{");
        self.loop_body(sc, out, &cond, None);
    }

    fn loop_body(
        &mut self,
        sc: &Scope,
        out: &mut Vec<crate::chardef::Controller>,
        cond: &str,
        step: Option<String>,
    ) {
        let body = self.braced();
        let break_slot = self.alloc_temp();
        let cont_slot = self.alloc_temp();
        self.var_set(break_slot, "0", sc, out);
        for _ in 0..LOOP_CAP {
            let mut inner = sc.clone();
            inner.conds.push(format!("!(var({break_slot}))"));
            if !cond.trim().is_empty() {
                inner.conds.push(format!("({cond})"));
            }
            let cont_reset = Scope {
                conds: inner.conds.clone(),
                persistent: sc.persistent,
                ignore_hitpause: sc.ignore_hitpause,
            };
            self.var_set(cont_slot, "0", &cont_reset, out);
            let mut sub = self.child(body.as_bytes());
            sub.loops.push(LoopCtx {
                break_slot,
                cont_slot,
            });
            sub.stmts(&inner, out, false);
            if let Some(step) = &step {
                let mut step_scope = sc.clone();
                step_scope.conds.push(format!("!(var({break_slot}))"));
                if !cond.trim().is_empty() {
                    step_scope.conds.push(format!("({cond})"));
                }
                self.assignment(step, &step_scope, out);
            }
        }
    }

    fn skip_braced(&mut self) {
        let _ = self.braced();
    }

    fn expand_call(&mut self, text: &str, sc: &Scope, out: &mut Vec<crate::chardef::Controller>) {
        let text = text.trim();
        let Some((name, rest)) = text.split_once('(') else {
            self.error(format!("zss: malformed call '{text}'"));
            return;
        };
        let Some(args) = rest.strip_suffix(')') else {
            self.error(format!("zss: malformed call '{text}'"));
            return;
        };
        let key = name.trim().to_ascii_lowercase();
        let Some(f) = self.funcs.get(&key).cloned() else {
            self.error(format!("zss: unknown function '{key}'"));
            return;
        };
        if self.depth >= 8 {
            self.error("zss: call nested too deep".into());
            return;
        }
        let args = split_top(args, b',');
        if args.len() != f.params.len() {
            self.error(format!(
                "zss: '{key}' expects {} arguments, got {}",
                f.params.len(),
                args.len()
            ));
            return;
        }
        let mut sub = self.child(f.body.as_bytes());
        let snapshot = sub.locals.borrow().clone();
        for (p, a) in f.params.iter().zip(args.iter()) {
            let key = p.trim().to_ascii_lowercase();
            let slot = sub.alloc_temp();
            sub.locals.borrow_mut().insert(key, slot);
            sub.var_set(slot, a, sc, out);
        }
        sub.stmts(sc, out, false);
        *sub.locals.borrow_mut() = snapshot;
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
            _ if c == sep && depth == 0 && !q => {
                out.push(s[st..i].to_string());
                st = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[st..].to_string());
    out.into_iter()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn key_values(body: &str) -> Vec<(String, String)> {
    let body: String = body
        .lines()
        .map(strip_comment)
        .collect::<Vec<_>>()
        .join("\n");
    split_top(&body, b';')
        .into_iter()
        .flat_map(|part| split_top(&part, b'\n'))
        .filter_map(|kv| {
            let (k, v) = kv.split_once(':')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
        })
        .collect()
}

fn strip_comment(l: &str) -> &str {
    let mut q = false;
    for (i, c) in l.char_indices() {
        match c {
            '"' => q = !q,
            '#' if !q => return &l[..i],
            _ => {}
        }
    }
    l
}

fn controller(name: &str, body: &str, sc: &Scope) -> Option<crate::chardef::Controller> {
    let mut entries = vec![("type".to_string(), name.to_string())];
    entries.extend(key_values(body));
    let sec = Section {
        name: format!("State {name}"),
        lname: format!("state {}", name.to_ascii_lowercase()),
        entries,
        lines: vec![],
    };
    let mut c = build_controller(&sec)?;
    c.trigger_all = sc
        .conds
        .iter()
        .map(|x| crate::chardef::safe_expr(x))
        .chain(c.trigger_all)
        .collect();
    if c.triggers.is_empty() {
        c.triggers = vec![vec![Expr::num(1.0)]];
    }
    if let Some(p) = sc.persistent {
        c.persistent = p;
    }
    if sc.ignore_hitpause {
        c.ignore_hitpause = true;
    }
    Some(c)
}

pub struct ZssReport {
    pub states: usize,
    pub unsupported: Vec<String>,
    pub errors: Vec<String>,
}

fn extract_functions(text: &str) -> (String, HashMap<String, FuncDef>) {
    let b = text.as_bytes();
    let (mut out, mut funcs) = (String::new(), HashMap::new());
    let (mut i, mut copy_from) = (0usize, 0usize);
    while i < b.len() {
        if b[i] != b'[' {
            i += 1;
            continue;
        }
        let Some(close) = text[i + 1..].find(']').map(|k| i + 1 + k) else {
            break;
        };
        let header = text[i + 1..close].trim();
        let lower = header.to_ascii_lowercase();
        if lower.starts_with("function") {
            let rest = header["function".len()..].trim();
            let (name, params) = match rest.split_once('(') {
                Some((n, p)) => (
                    n.trim().to_ascii_lowercase(),
                    p.strip_suffix(')')
                        .unwrap_or(p)
                        .split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<_>>(),
                ),
                None => (rest.to_ascii_lowercase(), vec![]),
            };
            let mut j = close + 1;
            while j < b.len() && (b[j] as char).is_ascii_whitespace() {
                j += 1;
            }
            if b.get(j) == Some(&b'{') {
                let st = j + 1;
                let mut depth = 1;
                j += 1;
                while j < b.len() {
                    match b[j] {
                        b'"' => {
                            j += 1;
                            while j < b.len() && b[j] != b'"' {
                                j += 1;
                            }
                        }
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                funcs.insert(
                    name,
                    FuncDef {
                        params,
                        body: String::from_utf8_lossy(&b[st..j.min(b.len())]).into_owned(),
                    },
                );
                out.push_str(&text[copy_from..i]);
                copy_from = (j + 1).min(b.len());
                i = copy_from;
                continue;
            }
        }
        i = close + 1;
    }
    out.push_str(&text[copy_from..]);
    (out, funcs)
}

pub fn parse_zss(
    text: &str,
    into: &mut HashMap<i32, StateDef>,
    override_existing: bool,
) -> ZssReport {
    let (source, funcs) = extract_functions(text);
    let mut r = Reader {
        s: source.as_bytes(),
        p: 0,
        unsupported: Rc::new(RefCell::new(vec![])),
        errors: Rc::new(RefCell::new(vec![])),
        locals: Rc::new(RefCell::new(HashMap::new())),
        next_local: Rc::new(Cell::new(LOCALS_TOP)),
        funcs: Rc::new(funcs),
        loops: vec![],
        extra: vec![],
        depth: 0,
    };
    let mut states = 0;
    loop {
        r.skip();
        match r.peek() {
            None => break,
            Some(b'[') => {}
            Some(_) => {
                r.p += 1;
                continue;
            }
        }
        r.p += 1;
        let header = r.until(b"]");
        r.p += 1;
        let parts = split_top(&header, b';');
        let head = parts.first().cloned().unwrap_or_default();
        let lhead = head.to_ascii_lowercase();
        let Some(rest) = lhead.strip_prefix("statedef") else {
            r.unsupported
                .borrow_mut()
                .push(lhead.split_whitespace().next().unwrap_or("").to_string());
            let mut sink = vec![];
            r.stmts(&Scope::default(), &mut sink, false);
            continue;
        };
        let id: i32 = rest.trim().parse().unwrap_or(i32::MIN);
        let mut ctrls = vec![];
        r.stmts(&Scope::default(), &mut ctrls, false);
        if id == i32::MIN || (into.contains_key(&id) && !override_existing && id >= 0) {
            continue;
        }
        states += 1;
        let params: Vec<(String, Param)> = parts
            .iter()
            .skip(1)
            .filter_map(|kv| kv.split_once(':'))
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), Param::new(v.trim())))
            .collect();
        if id < 0 && into.contains_key(&id) {
            into.get_mut(&id).unwrap().controllers.extend(ctrls);
        } else {
            into.insert(
                id,
                StateDef {
                    id,
                    params,
                    controllers: ctrls,
                },
            );
        }
    }
    let mut unsupported = r.unsupported.borrow().clone();
    let mut errors = r.errors.borrow().clone();
    unsupported.sort();
    unsupported.dedup();
    errors.sort();
    errors.dedup();
    ZssReport {
        states,
        unsupported,
        errors,
    }
}

#[cfg(test)]
mod tests;

pub fn parse_state_file(
    name: &str,
    text: &str,
    into: &mut HashMap<i32, StateDef>,
    override_existing: bool,
) {
    if name
        .trim()
        .trim_matches('"')
        .to_ascii_lowercase()
        .ends_with(".zss")
    {
        let mut tmp = HashMap::new();
        let r = parse_zss(text, &mut tmp, true);
        if !r.unsupported.is_empty() {
            eprintln!("zss {name}: unsupported {}", r.unsupported.join(", "));
        }
        if !r.errors.is_empty() {
            eprintln!("zss {name}: compile error: {}", r.errors.join("; "));
            return;
        }
        for (id, sd) in tmp {
            if id < 0 && into.contains_key(&id) {
                into.get_mut(&id)
                    .unwrap()
                    .controllers
                    .extend(sd.controllers);
            } else if override_existing || !into.contains_key(&id) {
                into.insert(id, sd);
            }
        }
    } else {
        crate::cns::parse_states(text, into, override_existing);
    }
}
