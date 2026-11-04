//! MUGEN expression language: tokenizer, parser and evaluator (tree-walking over a resolved AST).

#[derive(Clone, Debug, PartialEq)]
enum Tok { Num(String, bool), Id(String), Str(String), Op(&'static str), Eof }

const OPS: [&str; 26] = [":=", "**", "!=", "<=", ">=", "&&", "||", "^^", "=", "<", ">", "+", "-", "*", "/", "%", "!", "~", "&", "|", "^", "(", ")", "[", "]", ","];

fn tokenize(src: &str) -> Vec<Tok> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() { i += 1; continue; }
        if c == '"' {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'"' { j += 1; }
            out.push(Tok::Str(String::from_utf8_lossy(&b[i + 1..j]).into_owned()));
            i = j + 1;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && i + 1 < b.len() && (b[i + 1] as char).is_ascii_digit()) {
            let mut j = i;
            while j < b.len() && ((b[j] as char).is_ascii_digit() || b[j] == b'.') { j += 1; }
            let s = String::from_utf8_lossy(&b[i..j]).into_owned();
            let fl = s.contains('.');
            out.push(Tok::Num(s, fl));
            i = j;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i;
            while j < b.len() && ((b[j] as char).is_ascii_alphanumeric() || b[j] == b'_' || b[j] == b'.') { j += 1; }
            out.push(Tok::Id(String::from_utf8_lossy(&b[i..j]).into_owned()));
            i = j;
            continue;
        }
        if let Some(op) = OPS.iter().find(|o| src[i..].starts_with(**o)) {
            out.push(Tok::Op(op));
            i += op.len();
            continue;
        }
        i += 1;
    }
    out.push(Tok::Eof);
    out
}

#[derive(Clone, Debug)]
pub enum Node {
    Num(f64, bool),
    Bin(&'static str, Box<Node>, Box<Node>),
    Un(&'static str, Box<Node>),
    /// trigger name (lowercase), args, component (x/y/z), raw arg (const(...))
    Trig { name: String, args: Vec<Node>, comp: Option<char>, raw: Option<String> },
    Redir { kind: String, arg: Option<Box<Node>>, e: Box<Node> },
    CmpStr { name: String, neg: bool, values: Vec<String> },
    Interval { a: Box<Node>, neg: bool, lo: Box<Node>, hi: Box<Node>, lo_inc: bool, hi_inc: bool },
    Special { name: String, op: &'static str, a: Box<Node>, op2: Option<&'static str>, b: Option<Box<Node>> },
    Assign { name: String, idx: Box<Node>, v: Box<Node> },
}

const REDIRS: [&str; 10] = ["parent", "root", "helper", "target", "partner", "enemy", "enemynear", "playerid", "p2", "stateowner"];
const STRCMP: &[&str] = &["command", "statetype", "movetype", "p2statetype", "p2movetype", "name", "p1name", "p2name", "p3name", "p4name", "p5name", "p6name", "p7name", "p8name", "authorname", "hitdefattr", "physics", "stagevar", "p2command", "displayname", "helpername", "selfcommand"];
const COMPONENT: [&str; 13] = ["vel", "pos", "p2dist", "p2bodydist", "p1dist", "rootdist", "parentdist", "screenpos", "camerapos", "hitvel", "camerazoom", "scale", "offset"];
const RAWARG: [&str; 7] = ["const", "gethitvar", "stagevar", "const240p", "const480p", "const720p", "constcoord"];
const VARSPEC: &[&str] = &["explodvar", "projvar", "palfxvar", "clsnvar", "helpervar", "bgmvar", "spritevar", "animelemvar", "inputtime", "ikemenversion"];
const LEVELS: [&[&str]; 11] = [&["||"], &["^^"], &["&&"], &["|"], &["^"], &["&"], &["=", "!=", ":="], &["<", "<=", ">", ">="], &["+", "-"], &["*", "/", "%"], &["**"]];

struct Parser { t: Vec<Tok>, p: usize }

impl Parser {
    fn peek(&self, o: usize) -> &Tok { self.t.get(self.p + o).unwrap_or(self.t.last().unwrap()) }
    fn next(&mut self) -> Tok { let t = self.peek(0).clone(); self.p += 1; t }
    fn is_op(&self, v: &str, o: usize) -> bool { matches!(self.peek(o), Tok::Op(x) if *x == v) }
    fn expect(&mut self, v: &str) -> Result<(), String> { if self.is_op(v, 0) { self.p += 1; Ok(()) } else { Err(format!("expected {v}")) } }

    fn expr(&mut self, level: usize) -> Result<Node, String> {
        if level >= LEVELS.len() { return self.unary(); }
        let mut a = self.expr(level + 1)?;
        loop {
            let op = match self.peek(0) { Tok::Op(o) if LEVELS[level].contains(o) => *o, _ => break };
            self.p += 1;
            if (op == "=" || op == "!=") && (self.is_op("[", 0) || self.is_op("(", 0)) {
                if let Some(iv) = self.try_interval(a.clone(), op == "!=") { a = iv; continue; }
            }
            let b = if op == "**" { self.expr(level)? } else { self.expr(level + 1)? };
            a = Node::Bin(op, Box::new(a), Box::new(b));
        }
        Ok(a)
    }
    fn try_interval(&mut self, a: Node, neg: bool) -> Option<Node> {
        let save = self.p;
        let open = self.next();
        let r = (|| -> Result<Option<Node>, String> {
            let lo = self.expr(0)?;
            if !self.is_op(",", 0) { return Ok(None); }
            self.p += 1;
            let hi = self.expr(0)?;
            let close = self.next();
            let hi_inc = match close { Tok::Op("]") => true, Tok::Op(")") => false, _ => return Ok(None) };
            Ok(Some(Node::Interval { a: Box::new(a), neg, lo: Box::new(lo), hi: Box::new(hi), lo_inc: open == Tok::Op("["), hi_inc }))
        })();
        match r { Ok(Some(n)) => Some(n), _ => { self.p = save; None } }
    }
    fn unary(&mut self) -> Result<Node, String> {
        if let Tok::Op(o) = self.peek(0).clone() {
            if o == "!" || o == "~" || o == "-" || o == "+" {
                self.p += 1;
                let a = self.unary()?;
                return Ok(match (o, a) {
                    ("+", a) => a,
                    ("-", Node::Num(v, f)) => Node::Num(-v, f),
                    (o, a) => Node::Un(o, Box::new(a)),
                });
            }
        }
        self.primary()
    }
    fn var_spec(&mut self, name: String) -> Result<Node, String> {
        self.p += 1;
        let mut segments: Vec<Vec<Tok>> = vec![vec![]];
        let mut depth = 1;
        loop {
            let t = self.next();
            match &t {
                Tok::Eof => return Err("unterminated variable trigger".into()),
                Tok::Op("(") => depth += 1,
                Tok::Op(")") => { depth -= 1; if depth == 0 { break; } }
                Tok::Op(",") if depth == 1 => { segments.push(vec![]); continue; }
                _ => {}
            }
            if let Some(last) = segments.last_mut() { last.push(t); }
        }
        let spec = segments.pop().unwrap_or_default();
        let raw = spec.iter().filter_map(|t| match t { Tok::Id(s) | Tok::Num(s, _) => Some(if name == "inputtime" { s.clone() } else { s.to_ascii_lowercase() }), _ => None }).collect::<Vec<_>>().join(".");
        let mut args = vec![];
        for mut tokens in segments {
            tokens.push(Tok::Eof);
            args.push(Parser { t: tokens, p: 0 }.expr(0)?);
        }
        Ok(Node::Trig { name, args, comp: None, raw: Some(raw) })
    }
    fn primary(&mut self) -> Result<Node, String> {
        let t = self.next();
        let name = match t {
            Tok::Num(s, f) => return Ok(Node::Num(s.parse::<f64>().unwrap_or(0.0), f)),
            Tok::Str(s) => return Ok(Node::Trig { name: "__str".into(), args: vec![], comp: None, raw: Some(s) }),
            Tok::Op("(") => { let e = self.expr(0)?; self.expect(")")?; return Ok(e); }
            Tok::Id(s) => s.to_ascii_lowercase(),
            other => return Err(format!("unexpected {other:?}")),
        };
        if REDIRS.contains(&name.as_str()) {
            let save = self.p;
            let mut arg = None;
            if self.is_op("(", 0) {
                self.p += 1;
                arg = Some(Box::new(self.expr(0)?));
                self.expect(")")?;
            }
            if self.is_op(",", 0) {
                self.p += 1;
                let e = self.unary()?;
                return Ok(Node::Redir { kind: name, arg, e: Box::new(e) });
            }
            self.p = save;
        }
        if STRCMP.contains(&name.as_str()) && (self.is_op("=", 0) || self.is_op("!=", 0)) {
            let neg = self.next() == Tok::Op("!=");
            let mut values = vec![];
            match self.next() { Tok::Id(s) | Tok::Str(s) | Tok::Num(s, _) => values.push(s), _ => {} }
            if name == "hitdefattr" {
                while self.is_op(",", 0) {
                    if let Tok::Id(s) = self.peek(1).clone() {
                        let u = s.to_ascii_uppercase();
                        let bs = u.as_bytes();
                        if bs.len() == 2 && b"NSHAP".contains(&bs[0]) && b"ATP".contains(&bs[1]) { self.p += 2; values.push(s); continue; }
                    }
                    break;
                }
            }
            return Ok(Node::CmpStr { name, neg, values });
        }
        let is_special = matches!(name.as_str(), "animelem" | "timemod" | "projhit" | "projcontact" | "projguarded")
            || ((name.starts_with("projhit") || name.starts_with("projcontact") || name.starts_with("projguarded")) && name.chars().last().is_some_and(|c| c.is_ascii_digit()));
        if is_special && (self.is_op("=", 0) || self.is_op("!=", 0)) {
            let op = if self.next() == Tok::Op("=") { "=" } else { "!=" };
            let a = self.expr(8)?;
            if self.is_op(",", 0) {
                self.p += 1;
                let mut op2 = "=";
                if let Tok::Op(o) = self.peek(0).clone() { if ["=", "!=", "<", "<=", ">", ">="].contains(&o) { op2 = o; self.p += 1; } }
                let b = self.expr(8)?;
                return Ok(Node::Special { name, op, a: Box::new(a), op2: Some(op2), b: Some(Box::new(b)) });
            }
            return Ok(Node::Special { name, op, a: Box::new(a), op2: None, b: None });
        }
        if COMPONENT.contains(&name.as_str()) {
            if let Tok::Id(c) = self.peek(0).clone() {
                let lc = c.to_ascii_lowercase();
                if lc == "x" || lc == "y" || lc == "z" { self.p += 1; return Ok(Node::Trig { name, args: vec![], comp: lc.chars().next(), raw: None }); }
            }
        }
        if VARSPEC.contains(&name.as_str()) && self.is_op("(", 0) { return self.var_spec(name); }
        if self.is_op("(", 0) {
            self.p += 1;
            if RAWARG.contains(&name.as_str()) {
                let mut depth = 1; let mut raw = String::new();
                loop {
                    let x = self.next();
                    match &x {
                        Tok::Eof => break,
                        Tok::Op("(") => depth += 1,
                        Tok::Op(")") => { depth -= 1; if depth == 0 { break; } }
                        _ => {}
                    }
                    match x { Tok::Id(s) | Tok::Num(s, _) | Tok::Str(s) => raw.push_str(&s), Tok::Op(o) => raw.push_str(o), Tok::Eof => {} }
                }
                let raw = raw.to_ascii_lowercase();
                if name == "gethitvar" { crate::triggers::note_hit_var(&raw); }
                return Ok(Node::Trig { name, args: vec![], comp: None, raw: Some(raw) });
            }
            let mut args = vec![];
            if !self.is_op(")", 0) {
                args.push(self.expr(0)?);
                while self.is_op(",", 0) { self.p += 1; args.push(self.expr(0)?); }
            }
            self.expect(")")?;
            if matches!(name.as_str(), "var" | "fvar" | "sysvar" | "sysfvar") && self.is_op(":=", 0) {
                self.p += 1;
                let v = self.expr(0)?;
                return Ok(Node::Assign { name, idx: Box::new(args.into_iter().next().unwrap_or(Node::Num(0.0, false))), v: Box::new(v) });
            }
            return Ok(Node::Trig { name, args, comp: None, raw: None });
        }
        Ok(Node::Trig { name, args: vec![], comp: None, raw: None })
    }
}

pub fn parse(src: &str) -> Result<Node, String> {
    let mut p = Parser { t: tokenize(src), p: 0 };
    p.expr(0)
}

pub fn parse_list(src: &str) -> Result<Vec<Node>, String> {
    let mut p = Parser { t: tokenize(src), p: 0 };
    let mut out = vec![];
    if *p.peek(0) == Tok::Eof { return Ok(out); }
    out.push(p.expr(0)?);
    while p.is_op(",", 0) {
        p.p += 1;
        if *p.peek(0) == Tok::Eof { break; }
        out.push(p.expr(0)?);
    }
    Ok(out)
}

/// Whether an expression yields a float (MUGEN static typing: int ops truncate).
pub fn is_float(n: &Node) -> bool {
    match n {
        Node::Num(_, f) => *f,
        Node::Bin(op, a, b) => match *op { "+" | "-" | "*" | "/" | "**" => is_float(a) || is_float(b), ":=" => is_float(b), _ => false },
        Node::Un(op, a) => *op == "-" && is_float(a),
        Node::Trig { name, args, .. } => match name.as_str() {
            "vel" | "pos" | "screenpos" | "camerapos" | "p2dist" | "p1dist" | "p2bodydist" | "rootdist" | "parentdist" | "frontedgedist" | "backedgedist"
            | "frontedgebodydist" | "backedgebodydist" | "fvar" | "sysfvar" | "const" | "gethitvar" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "ln"
            | "log" | "sqrt" | "e" | "pi" | "const240p" | "const480p" | "const720p" | "camerazoom" | "backedge" | "frontedge" | "leftedge" | "rightedge" | "topedge" | "bottomedge" => true,
            "abs" | "min" | "max" | "clamp" => args.iter().any(is_float),
            "ifelse" | "cond" => args.iter().skip(1).any(is_float),
            _ => false,
        },
        Node::Redir { e, .. } => is_float(e),
        Node::Assign { name, .. } => name.contains('f'),
        _ => false,
    }
}

/// A compiled expression: the AST plus its static float-ness.
#[derive(Clone, Debug)]
pub struct Expr { pub node: Node, pub float: bool, pub code: crate::bytecode::Code }
impl Expr {
    pub fn new(node: Node) -> Self { let float = is_float(&node); let code = crate::bytecode::Code::compile(&node); Expr { node, float, code } }
    #[inline]
    pub fn eval<H: Host>(&self, h: &H) -> f64 { self.code.run(h) }
    pub fn num(v: f64) -> Self { Expr::new(Node::Num(v, v.fract() != 0.0)) }
}

/// Evaluation host: implemented by the match/fighter context.
pub trait Host {
    fn trigger(&self, name: &str, args: &[Node], comp: Option<char>, raw: Option<&str>) -> f64;
    fn redirect(&self, kind: &str, arg: Option<f64>) -> Option<Self> where Self: Sized;
    fn cmpstr(&self, name: &str, neg: bool, values: &[String]) -> bool;
    fn special(&self, name: &str, op: &str, a: f64, op2: Option<&str>, b: Option<f64>) -> bool;
    fn assign(&self, name: &str, idx: i32, v: f64) -> f64;
}

pub fn truthy(v: f64) -> bool { v != 0.0 && !v.is_nan() }
#[inline]
fn trunc(x: f64) -> f64 { x.trunc() }

pub fn eval<H: Host>(n: &Node, h: &H) -> f64 {
    match n {
        Node::Num(v, _) => *v,
        Node::Un(op, a) => {
            let v = eval(a, h);
            if v.is_nan() { return v; }
            match *op { "!" => (v == 0.0) as i32 as f64, "~" => !(trunc(v) as i32) as f64, _ => -v }
        }
        Node::Bin(op, a, b) => {
            let op = *op;
            if op == "&&" { return (truthy(eval(a, h)) && truthy(eval(b, h))) as i32 as f64; }
            if op == "||" { return (truthy(eval(a, h)) || truthy(eval(b, h))) as i32 as f64; }
            let x = eval(a, h);
            let y = eval(b, h);
            if x.is_nan() || y.is_nan() { return f64::NAN; }
            let fl = is_float(a) || is_float(b);
            let bo = |c: bool| c as i32 as f64;
            match op {
                "+" => x + y,
                "-" => x - y,
                "*" => if fl { x * y } else { (trunc(x) as i64).wrapping_mul(trunc(y) as i64) as i32 as f64 },
                "/" => if y == 0.0 { 0.0 } else if fl { x / y } else { trunc(x / y) },
                "%" => { let d = trunc(y) as i64; if d == 0 { 0.0 } else { ((trunc(x) as i64) % d) as f64 } }
                "**" => if fl { x.powf(y) } else { trunc(x.powf(y)) },
                "=" => bo(x == y),
                "!=" => bo(x != y),
                "<" => bo(x < y),
                "<=" => bo(x <= y),
                ">" => bo(x > y),
                ">=" => bo(x >= y),
                "^^" => bo((x != 0.0) != (y != 0.0)),
                "&" => ((trunc(x) as i64) & (trunc(y) as i64)) as f64,
                "|" => ((trunc(x) as i64) | (trunc(y) as i64)) as f64,
                "^" => ((trunc(x) as i64) ^ (trunc(y) as i64)) as f64,
                ":=" => y,
                _ => 0.0,
            }
        }
        Node::Interval { a, neg, lo, hi, lo_inc, hi_inc } => {
            let v = eval(a, h); let l = eval(lo, h); let u = eval(hi, h);
            if v.is_nan() || l.is_nan() || u.is_nan() { return f64::NAN; }
            let inside = (if *lo_inc { v >= l } else { v > l }) && (if *hi_inc { v <= u } else { v < u });
            (inside != *neg) as i32 as f64
        }
        Node::Redir { kind, arg, e } => {
            let a = arg.as_ref().map(|x| eval(x, h));
            match h.redirect(kind, a) { Some(t) => eval(e, &t), None => f64::NAN }
        }
        Node::CmpStr { name, neg, values } => h.cmpstr(name, *neg, values) as i32 as f64,
        Node::Special { name, op, a, op2, b } => {
            let av = eval(a, h);
            let bv = b.as_ref().map(|x| eval(x, h));
            h.special(name, op, av, *op2, bv) as i32 as f64
        }
        Node::Assign { name, idx, v } => { let i = eval(idx, h) as i32; let val = eval(v, h); h.assign(name, i, val) }
        Node::Trig { name, args, comp, raw } => match name.as_str() {
            "ifelse" | "cond" => {
                if args.len() < 3 { return 0.0; }
                if eval(&args[0], h) != 0.0 { eval(&args[1], h) } else { eval(&args[2], h) }
            }
            _ => h.trigger(name, args, *comp, raw.as_deref()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct H;
    impl Host for H {
        fn trigger(&self, name: &str, args: &[Node], comp: Option<char>, _raw: Option<&str>) -> f64 {
            match name { "time" => 7.0, "var" => [0.0, 5.0, 9.0][eval(&args[0], self) as usize], "vel" => if comp == Some('y') { -2.5 } else { 1.5 }, _ => 0.0 }
        }
        fn redirect(&self, _k: &str, _a: Option<f64>) -> Option<Self> { None }
        fn cmpstr(&self, name: &str, neg: bool, v: &[String]) -> bool { (name == "statetype" && v[0].eq_ignore_ascii_case("s")) != neg }
        fn special(&self, _: &str, _: &str, _: f64, _: Option<&str>, _: Option<f64>) -> bool { true }
        fn assign(&self, _: &str, _: i32, v: f64) -> f64 { v }
    }
    fn ev(s: &str) -> f64 { eval(&parse(s).unwrap(), &H) }
    #[test] fn ints() { assert_eq!(ev("7/2"), 3.0); assert_eq!(ev("7.0/2"), 3.5); assert_eq!(ev("-7/2"), -3.0); assert_eq!(ev("2**3"), 8.0); assert_eq!(ev("7%3"), 1.0); }
    #[test] fn logic() { assert_eq!(ev("1 + 2 * 3"), 7.0); assert_eq!(ev("time = 7 && var(1) = 5"), 1.0); assert_eq!(ev("!time || 0"), 0.0); }
    #[test] fn intervals() { assert_eq!(ev("time = [5, 7]"), 1.0); assert_eq!(ev("time = [5, 7)"), 0.0); assert_eq!(ev("time != (7, 9]"), 1.0); }
    #[test] fn misc() { assert_eq!(ev("vel y < 0"), 1.0); assert_eq!(ev("ifelse(time > 3, 10, 20)"), 10.0); assert_eq!(ev("statetype = S"), 1.0); assert_eq!(ev("statetype != A"), 1.0); assert!(ev("p2, time").is_nan()); assert!(!truthy(ev("p2, time = 0"))); assert!(!truthy(ev("p2, time != [1, 5]"))); assert!(!truthy(ev("!(p2, time)"))); assert_eq!(ev("(p2, time) || 1"), 1.0); }
}
