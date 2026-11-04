use crate::chardef::{next_uid, CharDef, Controller, StateDef};
use crate::cns::build_controller;
use crate::common::{common_def, flat_stage};
use crate::expr::{Host, Node};
use crate::matchs::Match;
use crate::triggers::Ctx;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use ume_formats::ini::parse_ini;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Controller,
    Parameter,
    Trigger,
    Redirection,
    StringTrigger,
    AssertFlag,
}

impl Category {
    fn label(self) -> &'static str {
        match self {
            Category::Controller => "unsupported controller",
            Category::Parameter => "unknown parameter",
            Category::Trigger => "unknown trigger",
            Category::Redirection => "unsupported redirection",
            Category::StringTrigger => "unsupported string trigger",
            Category::AssertFlag => "unknown AssertSpecial flag",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub category: Category,
    pub name: String,
    pub file: String,
    pub state: i32,
}

#[derive(Clone, Debug)]
pub struct SourceSpan {
    pub file: String,
    pub uids: Range<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct SourceSpans {
    spans: Vec<SourceSpan>,
}

impl SourceSpans {
    pub fn track<R>(&mut self, file: &str, parse: impl FnOnce() -> R) -> R {
        let start = next_uid();
        let result = parse();
        let end = next_uid();
        self.spans.push(SourceSpan { file: file.trim().trim_matches('"').to_string(), uids: start..end });
        result
    }

    pub fn file_of(&self, uid: u32) -> Option<&str> {
        self.spans.iter().rev().find(|s| s.uids.contains(&uid)).map(|s| s.file.as_str())
    }

    pub fn is_empty(&self) -> bool { self.spans.is_empty() }
}

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub findings: Vec<Finding>,
}

impl Diagnostics {
    pub fn is_clean(&self) -> bool { self.findings.is_empty() }

    pub fn distinct(&self, category: Category) -> BTreeMap<&str, Vec<&Finding>> {
        let mut out: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
        for f in self.findings.iter().filter(|f| f.category == category) { out.entry(f.name.as_str()).or_default().push(f); }
        out
    }

    pub fn names(&self, category: Category) -> Vec<String> {
        self.distinct(category).keys().map(|k| k.to_string()).collect()
    }

    pub fn count(&self, category: Category) -> usize { self.distinct(category).len() }

    pub fn summary(&self) -> String {
        let triggers = self.count(Category::Trigger) + self.count(Category::Redirection) + self.count(Category::StringTrigger);
        format!(
            "{} unsupported controllers, {} unknown triggers, {} unknown parameters, {} unknown AssertSpecial flags",
            self.count(Category::Controller), triggers, self.count(Category::Parameter), self.count(Category::AssertFlag)
        )
    }

    pub fn detail(&self) -> String {
        let mut out = String::new();
        for category in [Category::Controller, Category::Trigger, Category::Redirection, Category::StringTrigger, Category::Parameter, Category::AssertFlag] {
            for (name, hits) in self.distinct(category) {
                let places: BTreeSet<String> = hits.iter().map(|h| format!("{} state {}", h.file, h.state)).collect();
                let shown: Vec<&str> = places.iter().take(3).map(|s| s.as_str()).collect();
                let more = places.len().saturating_sub(shown.len());
                out.push_str(&format!("  {}: {} ({} uses; {}{})\n", category.label(), name, hits.len(), shown.join("; "), if more > 0 { format!("; +{more} more") } else { String::new() }));
            }
        }
        out
    }

    pub fn report(&self, character: &str, verbose: bool) -> String {
        if self.is_clean() { return String::new(); }
        let mut out = format!("{character}: {}\n", self.summary());
        if verbose { out.push_str(&self.detail()); }
        out
    }
}

pub fn audit(states: &HashMap<i32, StateDef>, spans: &SourceSpans) -> Diagnostics {
    let mut seen: BTreeSet<Finding> = BTreeSet::new();
    let mut ids: Vec<i32> = states.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let state = &states[&id];
        for controller in &state.controllers {
            let Some(file) = spans.file_of(controller.uid) else { continue };
            let mut push = |category: Category, name: String| {
                seen.insert(Finding { category, name, file: file.to_string(), state: id });
            };
            audit_controller(controller, &mut push);
        }
    }
    Diagnostics { findings: seen.into_iter().collect() }
}

fn audit_controller(c: &Controller, push: &mut impl FnMut(Category, String)) {
    if c.kind.starts_with("__") {
        for node in c.trigger_all.iter().chain(c.triggers.iter().flatten()).map(|e| &e.node) { audit_node(node, push); }
        for node in c.params.iter().flat_map(|(_, p)| p.exprs.iter().map(|e| &e.node)) { audit_node(node, push); }
        return;
    }
    let supported = controller_supported(&c.kind);
    if !supported { push(Category::Controller, c.kind.clone()); }
    for node in c.trigger_all.iter().chain(c.triggers.iter().flatten()).map(|e| &e.node) { audit_node(node, push); }
    for node in c.params.iter().flat_map(|(_, p)| p.exprs.iter().map(|e| &e.node)).filter(|n| !is_bare_word(n)) { audit_node(node, push); }
    if !supported { return; }
    let names: Vec<&str> = c.params.iter().map(|(k, _)| k.as_str()).filter(|k| !is_universal_param(k)).collect();
    if names.is_empty() { return; }
    let together = consumed_together(&c.kind, &names);
    for name in &names {
        if !together.contains(*name) && !parameter_consumed(&c.kind, name) { push(Category::Parameter, format!("{}.{}", c.kind, name)); }
    }
    if c.kind == "assertspecial" {
        for (_, p) in c.params.iter().filter(|(k, _)| k.starts_with("flag")) {
            let flag = p.raw.trim().to_ascii_lowercase();
            if !flag.is_empty() && !assert_flag_supported(&flag) { push(Category::AssertFlag, flag); }
        }
    }
}

fn is_bare_word(n: &Node) -> bool { matches!(n, Node::Trig { args, raw: None, comp: None, .. } if args.is_empty()) }

fn is_universal_param(name: &str) -> bool { matches!(name, "persistent" | "ignorehitpause" | "type") }

fn audit_node(n: &Node, push: &mut impl FnMut(Category, String)) {
    match n {
        Node::Num(..) => {}
        Node::Bin(_, a, b) => { audit_node(a, push); audit_node(b, push); }
        Node::Un(_, a) => audit_node(a, push),
        Node::Trig { name, args, raw, .. } => {
            if !matches!(name.as_str(), "__str" | "ifelse" | "cond") {
                if let Some(problem) = trigger_problem(name, raw.as_deref()) { push(Category::Trigger, problem); }
            }
            let takes_box_types = matches!(name.as_str(), "clsnoverlap" | "projclsnoverlap" | "clsnvar");
            for a in args {
                let is_box_type = matches!(a, Node::Trig { name, args, raw: None, comp: None } if args.is_empty() && matches!(name.as_str(), "clsn1" | "clsn2" | "size" | "dummy"));
                if !(takes_box_types && is_box_type) { audit_node(a, push); }
            }
        }
        Node::Redir { kind, arg, e } => {
            if !redirection_supported(kind) { push(Category::Redirection, kind.clone()); }
            if let Some(a) = arg { audit_node(a, push); }
            audit_node(e, push);
        }
        Node::CmpStr { name, .. } => { if !string_trigger_supported(name) { push(Category::StringTrigger, name.clone()); } }
        Node::Interval { a, lo, hi, .. } => { audit_node(a, push); audit_node(lo, push); audit_node(hi, push); }
        Node::Special { a, b, .. } => { audit_node(a, push); if let Some(b) = b { audit_node(b, push); } }
        Node::Assign { idx, v, .. } => { audit_node(idx, push); audit_node(v, push); }
    }
}

pub fn audit_cns(file: &str, text: &str) -> Diagnostics {
    let mut states = HashMap::new();
    let mut spans = SourceSpans::default();
    spans.track(file, || crate::cns::parse_states(text, &mut states, true));
    audit(&states, &spans)
}

pub fn audit_char(def: &CharDef, spans: &SourceSpans) -> Diagnostics { audit(&def.states, spans) }

thread_local! {
    static PROBING: Cell<bool> = const { Cell::new(false) };
    static NOTES: RefCell<Vec<(Category, String)>> = const { RefCell::new(Vec::new()) };
    static READS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static COMMON: RefCell<Option<Arc<CharDef>>> = const { RefCell::new(None) };
    static CACHE: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
    static TRIGGER_CACHE: RefCell<HashMap<String, Option<String>>> = RefCell::new(HashMap::new());
    static CONSUMED_CACHE: RefCell<HashMap<String, Arc<HashSet<String>>>> = RefCell::new(HashMap::new());
}

#[inline]
pub(crate) fn note_unsupported(category: Category, name: &str) {
    if PROBING.with(|p| p.get()) { NOTES.with(|n| n.borrow_mut().push((category, name.to_string()))); }
}

#[inline]
pub(crate) fn note_param_read(name: &str) {
    if PROBING.with(|p| p.get()) { READS.with(|r| { r.borrow_mut().insert(name.to_string()); }); }
}

fn probe_match() -> Match {
    let common = COMMON.with(|c| c.borrow_mut().get_or_insert_with(common_def).clone());
    let mut m = Match::new(common, None, flat_stage());
    let def = Arc::new(CharDef::empty("diagnostics-probe"));
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 1);
    m
}

struct ProbeOutcome {
    notes: Vec<(Category, String)>,
    reads: HashSet<String>,
    completed: bool,
}

fn probe(body: impl FnOnce(&mut Match)) -> ProbeOutcome {
    NOTES.with(|n| n.borrow_mut().clear());
    READS.with(|r| r.borrow_mut().clear());
    PROBING.with(|p| p.set(true));
    let mut m = probe_match();
    let completed = catch_unwind(AssertUnwindSafe(|| body(&mut m))).is_ok();
    PROBING.with(|p| p.set(false));
    ProbeOutcome { notes: NOTES.with(|n| std::mem::take(&mut *n.borrow_mut())), reads: READS.with(|r| std::mem::take(&mut *r.borrow_mut())), completed }
}

fn probe_controller(kind: &str, params: &[(&str, &str)]) -> Option<Controller> {
    let mut text = format!("[State 0, probe]\ntype = {kind}\n");
    for (k, v) in params { text.push_str(&format!("{k} = {v}\n")); }
    parse_ini(&text).first().and_then(build_controller)
}

fn run_controller(kind: &str, params: &[(&str, &str)]) -> ProbeOutcome {
    let Some(sc) = probe_controller(kind, params) else { return ProbeOutcome { notes: vec![], reads: HashSet::new(), completed: false } };
    probe(|m| crate::controllers::run(m, 0, &sc))
}

fn cached(key: String, compute: impl FnOnce() -> bool) -> bool {
    if let Some(v) = CACHE.with(|c| c.borrow().get(&key).copied()) { return v; }
    let v = compute();
    CACHE.with(|c| c.borrow_mut().insert(key, v));
    v
}

fn sane_name(name: &str) -> bool { !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '(' | ')' | ',' | ' ')) }

pub fn controller_supported(kind: &str) -> bool {
    if !sane_name(kind) { return false; }
    cached(format!("controller {kind}"), || {
        let out = run_controller(kind, &[]);
        !out.completed || !out.notes.iter().any(|(c, n)| *c == Category::Controller && n == kind)
    })
}

pub fn parameter_consumed(kind: &str, name: &str) -> bool {
    if !sane_name(name) { return false; }
    cached(format!("parameter {kind} {name}"), || {
        let out = run_controller(kind, &[(name, "0")]);
        !out.completed || out.reads.contains(name)
    })
}

fn consumed_together(kind: &str, names: &[&str]) -> Arc<HashSet<String>> {
    let mut sorted: Vec<&str> = names.iter().copied().filter(|n| sane_name(n)).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let key = format!("{kind} {}", sorted.join("|"));
    if let Some(v) = CONSUMED_CACHE.with(|c| c.borrow().get(&key).cloned()) { return v; }
    let params: Vec<(&str, &str)> = sorted.iter().map(|n| (*n, "0")).collect();
    let out = run_controller(kind, &params);
    let set = Arc::new(if out.completed { out.reads } else { sorted.iter().map(|s| s.to_string()).collect() });
    CONSUMED_CACHE.with(|c| c.borrow_mut().insert(key, set.clone()));
    set
}

pub fn assert_flag_supported(flag: &str) -> bool {
    if !sane_name(flag) { return false; }
    cached(format!("flag {flag}"), || {
        let out = run_controller("assertspecial", &[("flag", flag)]);
        !out.completed || !out.notes.iter().any(|(c, n)| *c == Category::AssertFlag && n == flag)
    })
}

pub fn trigger_supported(name: &str) -> bool { trigger_problem(name, None).is_none() }

pub fn trigger_problem(name: &str, raw: Option<&str>) -> Option<String> {
    if !sane_name(name) { return Some(name.to_string()); }
    if raw.is_some_and(|r| !sane_name(r)) { return None; }
    let key = format!("trigger {name} {}", raw.unwrap_or(""));
    if let Some(v) = TRIGGER_CACHE.with(|c| c.borrow().get(&key).cloned()) { return v; }
    let out = probe(|m| { Ctx { m, i: 0 }.trigger(name, &[Node::Num(0.0, false)], None, raw); });
    let sub_prefix = format!("{name}(");
    let problem = if !out.completed { None } else {
        out.notes.iter().filter(|(c, _)| *c == Category::Trigger).map(|(_, n)| n).find(|n| n.as_str() == name || (raw.is_some() && n.starts_with(&sub_prefix))).cloned()
    };
    TRIGGER_CACHE.with(|c| c.borrow_mut().insert(key, problem.clone()));
    problem
}

pub fn redirection_supported(kind: &str) -> bool {
    if !sane_name(kind) { return false; }
    cached(format!("redirection {kind}"), || {
        let out = probe(|m| { Ctx { m, i: 0 }.redirect(kind, Some(0.0)); });
        !out.completed || !out.notes.iter().any(|(c, n)| *c == Category::Redirection && n == kind)
    })
}

pub fn string_trigger_supported(name: &str) -> bool {
    if !sane_name(name) { return false; }
    cached(format!("string trigger {name}"), || {
        let out = probe(|m| { Ctx { m, i: 0 }.cmpstr(name, false, &["s".to_string()]); });
        !out.completed || !out.notes.iter().any(|(c, n)| *c == Category::StringTrigger && n == name)
    })
}
