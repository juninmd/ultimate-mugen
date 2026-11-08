use crate::expr::{eval, is_float, Host, Node};
mod fold;
pub use fold::truthy;
use fold::{binary, fold};

const STACK: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Const(f64),
    Host(u32),
    Trig0(u32),
    Trig1(u32),
    CmpStr(u32),
    Special {
        host: u32,
        has_b: bool,
    },
    Assign(u32),
    Redirect {
        kind: u32,
        has_arg: bool,
        len: u32,
    },
    Abs,
    Floor,
    Ceil,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Exp,
    Ln,
    Sqrt,
    Sign,
    Min,
    Max,
    Clamp,
    Not,
    BitNot,
    Neg,
    Add,
    Sub,
    MulI,
    MulF,
    DivI,
    DivF,
    Mod,
    PowI,
    PowF,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Xor,
    BitAnd,
    BitOr,
    BitXor,
    Second,
    Truthy,
    AndJump(u32),
    OrJump(u32),
    JumpIfZero(u32),
    Jump(u32),
    Interval {
        neg: bool,
        lo_inc: bool,
        hi_inc: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Code {
    ops: Vec<Op>,
    host: Vec<Node>,
    depth: usize,
}

impl Code {
    pub fn compile(n: &Node) -> Code {
        let mut c = Code::default();
        let folded = fold(n);
        let mut d = 0;
        c.emit(&folded, &mut d);
        c
    }

    pub fn constant(&self) -> Option<f64> {
        match self.ops.as_slice() {
            [Op::Const(v)] => Some(*v),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    fn push(&mut self, op: Op, d: &mut usize, delta: isize) {
        self.ops.push(op);
        *d = (*d as isize + delta) as usize;
        self.depth = self.depth.max(*d);
    }

    fn host_call(&mut self, n: &Node, d: &mut usize) {
        self.host.push(n.clone());
        self.push(Op::Host(self.host.len() as u32 - 1), d, 1);
    }

    fn patch(&mut self, at: usize) {
        let to = self.ops.len() as u32;
        match &mut self.ops[at] {
            Op::AndJump(t) | Op::OrJump(t) | Op::JumpIfZero(t) | Op::Jump(t) => *t = to,
            _ => {}
        }
    }

    fn emit(&mut self, n: &Node, d: &mut usize) {
        match n {
            Node::Num(v, _) => self.push(Op::Const(*v), d, 1),
            Node::Un(op, a) => {
                self.emit(a, d);
                let o = match *op {
                    "!" => Op::Not,
                    "~" => Op::BitNot,
                    _ => Op::Neg,
                };
                self.push(o, d, 0);
            }
            Node::Bin(op, a, b) if *op == "&&" || *op == "||" => {
                self.emit(a, d);
                let at = self.ops.len();
                self.push(
                    if *op == "&&" {
                        Op::AndJump(0)
                    } else {
                        Op::OrJump(0)
                    },
                    d,
                    -1,
                );
                self.emit(b, d);
                self.push(Op::Truthy, d, 0);
                self.patch(at);
            }
            Node::Bin(op, a, b) => {
                let fl = is_float(a) || is_float(b);
                let o = match *op {
                    "+" => Op::Add,
                    "-" => Op::Sub,
                    "*" => {
                        if fl {
                            Op::MulF
                        } else {
                            Op::MulI
                        }
                    }
                    "/" => {
                        if fl {
                            Op::DivF
                        } else {
                            Op::DivI
                        }
                    }
                    "%" => Op::Mod,
                    "**" => {
                        if fl {
                            Op::PowF
                        } else {
                            Op::PowI
                        }
                    }
                    "=" => Op::Eq,
                    "!=" => Op::Ne,
                    "<" => Op::Lt,
                    "<=" => Op::Le,
                    ">" => Op::Gt,
                    ">=" => Op::Ge,
                    "^^" => Op::Xor,
                    "&" => Op::BitAnd,
                    "|" => Op::BitOr,
                    "^" => Op::BitXor,
                    ":=" => Op::Second,
                    _ => {
                        self.push(Op::Const(0.0), d, 1);
                        return;
                    }
                };
                self.emit(a, d);
                self.emit(b, d);
                self.push(o, d, -1);
            }
            Node::Interval {
                a,
                neg,
                lo,
                hi,
                lo_inc,
                hi_inc,
            } => {
                self.emit(a, d);
                self.emit(lo, d);
                self.emit(hi, d);
                self.push(
                    Op::Interval {
                        neg: *neg,
                        lo_inc: *lo_inc,
                        hi_inc: *hi_inc,
                    },
                    d,
                    -2,
                );
            }
            Node::Trig { name, args, .. } if name == "ifelse" || name == "cond" => {
                if args.len() < 3 {
                    self.push(Op::Const(0.0), d, 1);
                    return;
                }
                self.emit(&args[0], d);
                let jz = self.ops.len();
                self.push(Op::JumpIfZero(0), d, -1);
                self.emit(&args[1], d);
                let jmp = self.ops.len();
                self.push(Op::Jump(0), d, -1);
                self.patch(jz);
                self.emit(&args[2], d);
                self.patch(jmp);
            }
            Node::Trig {
                name,
                args,
                comp,
                raw: None,
            } if args.len() == 1 && !matches!(name.as_str(), "isasserted" | "map" | "teammap") => {
                self.emit(&args[0], d);
                let o = match name.as_str() {
                    "abs" => Some(Op::Abs),
                    "floor" => Some(Op::Floor),
                    "ceil" => Some(Op::Ceil),
                    "sin" => Some(Op::Sin),
                    "cos" => Some(Op::Cos),
                    "tan" => Some(Op::Tan),
                    "asin" => Some(Op::Asin),
                    "acos" => Some(Op::Acos),
                    "atan" => Some(Op::Atan),
                    "exp" => Some(Op::Exp),
                    "ln" => Some(Op::Ln),
                    "sqrt" => Some(Op::Sqrt),
                    "sign" => Some(Op::Sign),
                    _ => None,
                };
                if let Some(op) = o {
                    self.push(op, d, 0);
                } else {
                    let k = self.host.len() as u32;
                    self.host.push(Node::Trig {
                        name: name.clone(),
                        args: vec![],
                        comp: *comp,
                        raw: None,
                    });
                    self.push(Op::Trig1(k), d, 0);
                }
            }
            Node::Trig {
                name,
                args,
                raw: None,
                ..
            } if (name == "min" || name == "max") && args.len() == 2 => {
                self.emit(&args[0], d);
                self.emit(&args[1], d);
                self.push(if name == "min" { Op::Min } else { Op::Max }, d, -1);
            }
            Node::Trig {
                name,
                args,
                raw: None,
                ..
            } if name == "clamp" && args.len() == 3 => {
                self.emit(&args[0], d);
                self.emit(&args[1], d);
                self.emit(&args[2], d);
                self.push(Op::Clamp, d, -2);
            }
            Node::Trig {
                name,
                args,
                raw: None,
                ..
            } if (name == "e" || name == "pi" || name == "__str") && args.is_empty() => {
                let v = match name.as_str() {
                    "e" => std::f64::consts::E,
                    "pi" => std::f64::consts::PI,
                    _ => 0.0,
                };
                self.push(Op::Const(v), d, 1);
            }
            Node::Trig { args, .. } if args.is_empty() => {
                let k = self.host.len() as u32;
                self.host.push(n.clone());
                self.push(Op::Trig0(k), d, 1);
            }
            Node::CmpStr { .. } => {
                let k = self.host.len() as u32;
                self.host.push(n.clone());
                self.push(Op::CmpStr(k), d, 1);
            }
            Node::Special {
                name,
                op,
                a,
                op2,
                b,
            } => {
                self.emit(a, d);
                let has_b = b.is_some();
                if let Some(bb) = b {
                    self.emit(bb, d);
                }
                let k = self.host.len() as u32;
                self.host.push(Node::Special {
                    name: name.clone(),
                    op,
                    a: Box::new(Node::Num(0.0, false)),
                    op2: *op2,
                    b: None,
                });
                self.push(
                    Op::Special { host: k, has_b },
                    d,
                    if has_b { -1 } else { 0 },
                );
            }
            Node::Assign { name, idx, v } => {
                self.emit(idx, d);
                self.emit(v, d);
                let k = self.host.len() as u32;
                self.host.push(Node::Trig {
                    name: name.clone(),
                    args: vec![],
                    comp: None,
                    raw: None,
                });
                self.push(Op::Assign(k), d, -1);
            }
            Node::Redir { kind, arg, e } => {
                let has_arg = arg.is_some();
                if let Some(a) = arg {
                    self.emit(a, d);
                }
                let k = self.host.len() as u32;
                self.host.push(Node::Trig {
                    name: kind.clone(),
                    args: vec![],
                    comp: None,
                    raw: None,
                });
                let sub = Code::compile(e);
                let len = sub.ops.len() as u32;
                self.push(
                    Op::Redirect {
                        kind: k,
                        has_arg,
                        len,
                    },
                    d,
                    if has_arg { 0 } else { 1 },
                );
                self.depth = self.depth.max(*d + sub.depth);
                let base_op = self.ops.len() as u32;
                let base_host = self.host.len() as u32;
                self.host.extend(sub.host);
                for op in sub.ops {
                    self.ops.push(match op {
                        Op::Host(h) => Op::Host(h + base_host),
                        Op::Trig0(h) => Op::Trig0(h + base_host),
                        Op::Trig1(h) => Op::Trig1(h + base_host),
                        Op::CmpStr(h) => Op::CmpStr(h + base_host),
                        Op::Special { host, has_b } => Op::Special {
                            host: host + base_host,
                            has_b,
                        },
                        Op::Assign(h) => Op::Assign(h + base_host),
                        Op::Redirect { kind, has_arg, len } => Op::Redirect {
                            kind: kind + base_host,
                            has_arg,
                            len,
                        },
                        Op::AndJump(t) => Op::AndJump(t + base_op),
                        Op::OrJump(t) => Op::OrJump(t + base_op),
                        Op::JumpIfZero(t) => Op::JumpIfZero(t + base_op),
                        Op::Jump(t) => Op::Jump(t + base_op),
                        other => other,
                    });
                }
            }
            _ => self.host_call(n, d),
        }
    }

    #[inline]
    pub fn run<H: Host>(&self, h: &H) -> f64 {
        if let Some(&first) = self.ops.first() {
            if self.ops.len() == 1 {
                match first {
                    Op::Const(v) => return v,
                    Op::Trig0(i) => {
                        if let Node::Trig {
                            name, comp, raw, ..
                        } = &self.host[i as usize]
                        {
                            return h.trigger0(name, *comp, raw.as_deref());
                        }
                        return 0.0;
                    }
                    Op::Host(i) => return eval(&self.host[i as usize], h),
                    _ => {}
                }
            }
        } else {
            return 0.0;
        }
        if self.depth > STACK {
            return 0.0;
        }
        if self.ops.len() == 2 {
            match (self.ops[0], self.ops[1]) {
                (Op::Trig0(i), Op::Not) => {
                    let v = if let Node::Trig {
                        name, comp, raw, ..
                    } = &self.host[i as usize]
                    {
                        h.trigger0(name, *comp, raw.as_deref())
                    } else {
                        0.0
                    };
                    return if v.is_nan() {
                        f64::NAN
                    } else if v == 0.0 {
                        1.0
                    } else {
                        0.0
                    };
                }
                (Op::Const(v), Op::Neg) => return -v,
                (Op::Const(v), Op::Not) => return if v == 0.0 { 1.0 } else { 0.0 },
                (Op::Host(i), Op::Not) => {
                    let v = eval(&self.host[i as usize], h);
                    return if v.is_nan() {
                        f64::NAN
                    } else if v == 0.0 {
                        1.0
                    } else {
                        0.0
                    };
                }
                (Op::Const(arg), Op::Trig1(i)) => {
                    return match &self.host[i as usize] {
                        Node::Trig {
                            name, comp, raw, ..
                        } => h.trigger1(name, arg, *comp, raw.as_deref()),
                        _ => 0.0,
                    };
                }
                _ => {}
            }
        }
        if self.ops.len() == 3 {
            let (op0, op1, op2) = (self.ops[0], self.ops[1], self.ops[2]);
            if let (Op::Const(arg), Op::Trig1(i), Op::Not) = (op0, op1, op2) {
                let v = match &self.host[i as usize] {
                    Node::Trig {
                        name, comp, raw, ..
                    } => h.trigger1(name, arg, *comp, raw.as_deref()),
                    _ => 0.0,
                };
                return if v.is_nan() {
                    f64::NAN
                } else if v == 0.0 {
                    1.0
                } else {
                    0.0
                };
            }
            if let (Op::Trig0(i), Op::Const(c)) = (op0, op1) {
                let v = if let Node::Trig {
                    name, comp, raw, ..
                } = &self.host[i as usize]
                {
                    h.trigger0(name, *comp, raw.as_deref())
                } else {
                    0.0
                };
                if v.is_nan() || c.is_nan() {
                    return f64::NAN;
                }
                return match op2 {
                    Op::Eq => (v == c) as i32 as f64,
                    Op::Ne => (v != c) as i32 as f64,
                    Op::Gt => (v > c) as i32 as f64,
                    Op::Ge => (v >= c) as i32 as f64,
                    Op::Lt => (v < c) as i32 as f64,
                    Op::Le => (v <= c) as i32 as f64,
                    Op::Add => v + c,
                    Op::Sub => v - c,
                    _ => self.run_slice(h, 0, 3),
                };
            } else if let (Op::Const(c), Op::Trig0(i)) = (op0, op1) {
                let v = if let Node::Trig {
                    name, comp, raw, ..
                } = &self.host[i as usize]
                {
                    h.trigger0(name, *comp, raw.as_deref())
                } else {
                    0.0
                };
                if v.is_nan() || c.is_nan() {
                    return f64::NAN;
                }
                return match op2 {
                    Op::Eq => (c == v) as i32 as f64,
                    Op::Ne => (c != v) as i32 as f64,
                    Op::Gt => (c > v) as i32 as f64,
                    Op::Ge => (c >= v) as i32 as f64,
                    Op::Lt => (c < v) as i32 as f64,
                    Op::Le => (c <= v) as i32 as f64,
                    Op::Add => c + v,
                    Op::Sub => c - v,
                    _ => self.run_slice(h, 0, 3),
                };
            } else if let (Op::Trig0(i), Op::Trig0(j)) = (op0, op1) {
                let v0 = if let Node::Trig {
                    name, comp, raw, ..
                } = &self.host[i as usize]
                {
                    h.trigger0(name, *comp, raw.as_deref())
                } else {
                    0.0
                };
                let v1 = if let Node::Trig {
                    name, comp, raw, ..
                } = &self.host[j as usize]
                {
                    h.trigger0(name, *comp, raw.as_deref())
                } else {
                    0.0
                };
                if v0.is_nan() || v1.is_nan() {
                    return f64::NAN;
                }
                return match op2 {
                    Op::Eq => (v0 == v1) as i32 as f64,
                    Op::Ne => (v0 != v1) as i32 as f64,
                    Op::Gt => (v0 > v1) as i32 as f64,
                    Op::Ge => (v0 >= v1) as i32 as f64,
                    Op::Lt => (v0 < v1) as i32 as f64,
                    Op::Le => (v0 <= v1) as i32 as f64,
                    Op::Add => v0 + v1,
                    Op::Sub => v0 - v1,
                    _ => self.run_slice(h, 0, 3),
                };
            }
        }
        if self.ops.len() == 4 {
            let (op0, op1, op2, op3) = (self.ops[0], self.ops[1], self.ops[2], self.ops[3]);
            if let (Op::Const(arg), Op::Trig1(i), Op::Const(c)) = (op0, op1, op2) {
                let v = match &self.host[i as usize] {
                    Node::Trig {
                        name, comp, raw, ..
                    } => h.trigger1(name, arg, *comp, raw.as_deref()),
                    _ => 0.0,
                };
                if v.is_nan() || c.is_nan() {
                    return f64::NAN;
                }
                return match op3 {
                    Op::Eq => (v == c) as i32 as f64,
                    Op::Ne => (v != c) as i32 as f64,
                    Op::Gt => (v > c) as i32 as f64,
                    Op::Ge => (v >= c) as i32 as f64,
                    Op::Lt => (v < c) as i32 as f64,
                    Op::Le => (v <= c) as i32 as f64,
                    Op::Add => v + c,
                    Op::Sub => v - c,
                    _ => self.run_slice(h, 0, 4),
                };
            } else if let (Op::Const(c), Op::Const(arg), Op::Trig1(i)) = (op0, op1, op2) {
                let v = match &self.host[i as usize] {
                    Node::Trig {
                        name, comp, raw, ..
                    } => h.trigger1(name, arg, *comp, raw.as_deref()),
                    _ => 0.0,
                };
                if v.is_nan() || c.is_nan() {
                    return f64::NAN;
                }
                return match op3 {
                    Op::Eq => (c == v) as i32 as f64,
                    Op::Ne => (c != v) as i32 as f64,
                    Op::Gt => (c > v) as i32 as f64,
                    Op::Ge => (c >= v) as i32 as f64,
                    Op::Lt => (c < v) as i32 as f64,
                    Op::Le => (c <= v) as i32 as f64,
                    Op::Add => c + v,
                    Op::Sub => c - v,
                    _ => self.run_slice(h, 0, 4),
                };
            }
        }

        self.run_slice(h, 0, self.ops.len())
    }

    fn run_slice<H: Host>(&self, h: &H, start: usize, end: usize) -> f64 {
        let mut s = [0.0f64; STACK];
        let mut sp = 0usize;
        let mut pc = start;
        let ops = self.ops.as_slice();
        let host = self.host.as_slice();
        let ib = |c: bool| c as i32 as f64;
        let it = |x: f64| x.trunc() as i64;
        while pc < end {
            match ops[pc] {
                Op::Const(v) => {
                    s[sp] = v;
                    sp += 1;
                }
                Op::Host(i) => {
                    s[sp] = eval(&host[i as usize], h);
                    sp += 1;
                }
                Op::Trig0(i) => {
                    if let Node::Trig {
                        name, comp, raw, ..
                    } = &host[i as usize]
                    {
                        s[sp] = h.trigger0(name, *comp, raw.as_deref());
                    } else {
                        s[sp] = 0.0;
                    }
                    sp += 1;
                }
                Op::Trig1(i) => {
                    s[sp - 1] = match &self.host[i as usize] {
                        Node::Trig {
                            name, comp, raw, ..
                        } => h.trigger1(name, s[sp - 1], *comp, raw.as_deref()),
                        _ => 0.0,
                    };
                }
                Op::CmpStr(i) => {
                    s[sp] = match &host[i as usize] {
                        Node::CmpStr { name, neg, values } => ib(h.cmpstr(name, *neg, values)),
                        _ => 0.0,
                    };
                    sp += 1;
                }
                Op::Special {
                    host: host_idx,
                    has_b,
                } => {
                    let bv = if has_b {
                        sp -= 1;
                        Some(s[sp])
                    } else {
                        None
                    };
                    let av = s[sp - 1];
                    s[sp - 1] = match &host[host_idx as usize] {
                        Node::Special { name, op, op2, .. } => {
                            ib(h.special(name, op, av, *op2, bv))
                        }
                        _ => 0.0,
                    };
                }
                Op::Assign(i) => {
                    sp -= 1;
                    let (idx, val) = (s[sp - 1] as i32, s[sp]);
                    s[sp - 1] = match &host[i as usize] {
                        Node::Trig { name, .. } => h.assign(name, idx, val),
                        _ => val,
                    };
                }
                Op::Redirect { kind, has_arg, len } => {
                    let arg = if has_arg {
                        sp -= 1;
                        Some(s[sp])
                    } else {
                        None
                    };
                    let name = match &host[kind as usize] {
                        Node::Trig { name, .. } => name.as_str(),
                        _ => "",
                    };
                    let sub_start = pc + 1;
                    let sub_end = sub_start + len as usize;
                    s[sp] = match h.redirect(name, arg) {
                        Some(t) => self.run_slice(&t, sub_start, sub_end),
                        None => f64::NAN,
                    };
                    sp += 1;
                    pc = sub_end;
                    continue;
                }
                Op::Abs => s[sp - 1] = s[sp - 1].abs(),
                Op::Floor => s[sp - 1] = s[sp - 1].floor(),
                Op::Ceil => s[sp - 1] = s[sp - 1].ceil(),
                Op::Sin => s[sp - 1] = s[sp - 1].sin(),
                Op::Cos => s[sp - 1] = s[sp - 1].cos(),
                Op::Tan => s[sp - 1] = s[sp - 1].tan(),
                Op::Asin => s[sp - 1] = s[sp - 1].asin(),
                Op::Acos => s[sp - 1] = s[sp - 1].acos(),
                Op::Atan => s[sp - 1] = s[sp - 1].atan(),
                Op::Exp => s[sp - 1] = s[sp - 1].exp(),
                Op::Ln => s[sp - 1] = s[sp - 1].ln(),
                Op::Sqrt => s[sp - 1] = s[sp - 1].sqrt(),
                Op::Sign => s[sp - 1] = s[sp - 1].signum(),
                Op::Min => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        x.min(y)
                    };
                }
                Op::Max => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        x.max(y)
                    };
                }
                Op::Clamp => {
                    sp -= 2;
                    let (v, lo, hi) = (s[sp - 1], s[sp], s[sp + 1]);
                    s[sp - 1] = if v.is_nan() || lo.is_nan() || hi.is_nan() {
                        f64::NAN
                    } else {
                        v.max(lo).min(hi)
                    };
                }
                Op::Not => {
                    if !s[sp - 1].is_nan() {
                        s[sp - 1] = ib(s[sp - 1] == 0.0)
                    }
                }
                Op::BitNot => {
                    if !s[sp - 1].is_nan() {
                        s[sp - 1] = !(s[sp - 1].trunc() as i32) as f64
                    }
                }
                Op::Neg => s[sp - 1] = -s[sp - 1],
                Op::Truthy => s[sp - 1] = ib(truthy(s[sp - 1])),
                Op::AndJump(t) => {
                    sp -= 1;
                    if !truthy(s[sp]) {
                        s[sp] = 0.0;
                        sp += 1;
                        pc = t as usize;
                        continue;
                    }
                }
                Op::OrJump(t) => {
                    sp -= 1;
                    if truthy(s[sp]) {
                        s[sp] = 1.0;
                        sp += 1;
                        pc = t as usize;
                        continue;
                    }
                }
                Op::JumpIfZero(t) => {
                    sp -= 1;
                    if s[sp] == 0.0 {
                        pc = t as usize;
                        continue;
                    }
                }
                Op::Jump(t) => {
                    pc = t as usize;
                    continue;
                }
                Op::Interval {
                    neg,
                    lo_inc,
                    hi_inc,
                } => {
                    sp -= 2;
                    let (v, l, u) = (s[sp - 1], s[sp], s[sp + 1]);
                    if v.is_nan() || l.is_nan() || u.is_nan() {
                        s[sp - 1] = f64::NAN;
                        pc += 1;
                        continue;
                    }
                    let inside = (if lo_inc { v >= l } else { v > l })
                        && (if hi_inc { v <= u } else { v < u });
                    s[sp - 1] = ib(inside != neg);
                }
                Op::Add => {
                    sp -= 1;
                    s[sp - 1] += s[sp];
                }
                Op::Sub => {
                    sp -= 1;
                    s[sp - 1] -= s[sp];
                }
                Op::MulF => {
                    sp -= 1;
                    s[sp - 1] *= s[sp];
                }
                Op::MulI => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        it(x).wrapping_mul(it(y)) as i32 as f64
                    };
                }
                Op::DivF => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if y == 0.0 { 0.0 } else { x / y };
                }
                Op::DivI => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if y == 0.0 { 0.0 } else { (x / y).trunc() };
                }
                Op::Eq => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x == y)
                    };
                }
                Op::Ne => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x != y)
                    };
                }
                Op::Lt => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x < y)
                    };
                }
                Op::Le => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x <= y)
                    };
                }
                Op::Gt => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x > y)
                    };
                }
                Op::Ge => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib(x >= y)
                    };
                }
                Op::BitAnd => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        (it(x) & it(y)) as f64
                    };
                }
                Op::BitOr => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        (it(x) | it(y)) as f64
                    };
                }
                Op::BitXor => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        (it(x) ^ it(y)) as f64
                    };
                }
                Op::Xor => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        ib((x != 0.0) != (y != 0.0))
                    };
                }
                op => {
                    sp -= 1;
                    let (x, y) = (s[sp - 1], s[sp]);
                    s[sp - 1] = if x.is_nan() || y.is_nan() {
                        f64::NAN
                    } else {
                        binary(op, x, y, it)
                    };
                }
            }
            pc += 1;
        }
        if sp == 0 {
            0.0
        } else {
            s[sp - 1]
        }
    }
}

#[cfg(test)]
mod tests;
