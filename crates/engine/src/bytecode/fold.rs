use super::Op;
use crate::expr::{eval, is_float, Host, Node};

#[inline]
pub fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

#[inline]
pub(super) fn binary(op: Op, x: f64, y: f64, it: impl Fn(f64) -> i64) -> f64 {
    let ib = |c: bool| c as i32 as f64;
    match op {
        Op::Add => x + y,
        Op::Sub => x - y,
        Op::MulF => x * y,
        Op::MulI => it(x).wrapping_mul(it(y)) as i32 as f64,
        Op::DivF => {
            if y == 0.0 {
                0.0
            } else {
                x / y
            }
        }
        Op::DivI => {
            if y == 0.0 {
                0.0
            } else {
                (x / y).trunc()
            }
        }
        Op::Mod => {
            let m = it(y);
            if m == 0 {
                0.0
            } else {
                (it(x) % m) as f64
            }
        }
        Op::PowF => x.powf(y),
        Op::PowI => x.powf(y).trunc(),
        Op::Eq => ib(x == y),
        Op::Ne => ib(x != y),
        Op::Lt => ib(x < y),
        Op::Le => ib(x <= y),
        Op::Gt => ib(x > y),
        Op::Ge => ib(x >= y),
        Op::Xor => ib((x != 0.0) != (y != 0.0)),
        Op::BitAnd => (it(x) & it(y)) as f64,
        Op::BitOr => (it(x) | it(y)) as f64,
        Op::BitXor => (it(x) ^ it(y)) as f64,
        _ => y,
    }
}

struct NoHost;
impl Host for NoHost {
    fn trigger(&self, _: &str, _: &[Node], _: Option<char>, _: Option<&str>) -> f64 {
        0.0
    }
    fn redirect(&self, _: &str, _: Option<f64>) -> Option<Self> {
        None
    }
    fn cmpstr(&self, _: &str, _: bool, _: &[String]) -> bool {
        false
    }
    fn special(&self, _: &str, _: &str, _: f64, _: Option<&str>, _: Option<f64>) -> bool {
        false
    }
    fn assign(&self, _: &str, _: i32, v: f64) -> f64 {
        v
    }
}

fn num(n: &Node) -> Option<f64> {
    if let Node::Num(v, _) = n {
        Some(*v)
    } else {
        None
    }
}

pub fn fold(n: &Node) -> Node {
    match n {
        Node::Un(op, a) => {
            let a = fold(a);
            let constant = num(&a).is_some();
            let r = Node::Un(op, Box::new(a));
            if constant {
                Node::Num(eval(&r, &NoHost), is_float(&r))
            } else {
                r
            }
        }
        Node::Bin(op, a, b) => {
            let (a, b) = (fold(a), fold(b));
            let both = num(&a).is_some() && num(&b).is_some();
            let r = Node::Bin(op, Box::new(a), Box::new(b));
            if both && *op != ":=" {
                Node::Num(eval(&r, &NoHost), is_float(&r))
            } else {
                r
            }
        }
        Node::Interval {
            a,
            neg,
            lo,
            hi,
            lo_inc,
            hi_inc,
        } => {
            let r = Node::Interval {
                a: Box::new(fold(a)),
                neg: *neg,
                lo: Box::new(fold(lo)),
                hi: Box::new(fold(hi)),
                lo_inc: *lo_inc,
                hi_inc: *hi_inc,
            };
            if let Node::Interval { a, lo, hi, .. } = &r {
                if num(a).is_some() && num(lo).is_some() && num(hi).is_some() {
                    return Node::Num(eval(&r, &NoHost), false);
                }
            }
            r
        }
        Node::Trig {
            name,
            args,
            comp,
            raw,
        } if name == "ifelse" || name == "cond" => {
            let args: Vec<Node> = args.iter().map(fold).collect();
            if args.len() >= 3 {
                if let Some(c) = num(&args[0]) {
                    return if c != 0.0 {
                        args[1].clone()
                    } else {
                        args[2].clone()
                    };
                }
            }
            Node::Trig {
                name: name.clone(),
                args,
                comp: *comp,
                raw: raw.clone(),
            }
        }
        Node::Trig {
            name,
            args,
            comp,
            raw,
        } => Node::Trig {
            name: name.clone(),
            args: args.iter().map(fold).collect(),
            comp: *comp,
            raw: raw.clone(),
        },
        Node::Redir { kind, arg, e } => Node::Redir {
            kind: kind.clone(),
            arg: arg.as_ref().map(|x| Box::new(fold(x))),
            e: Box::new(fold(e)),
        },
        Node::Special {
            name,
            op,
            a,
            op2,
            b,
        } => Node::Special {
            name: name.clone(),
            op,
            a: Box::new(fold(a)),
            op2: *op2,
            b: b.as_ref().map(|x| Box::new(fold(x))),
        },
        Node::Assign { name, idx, v } => Node::Assign {
            name: name.clone(),
            idx: Box::new(fold(idx)),
            v: Box::new(fold(v)),
        },
        other => other.clone(),
    }
}
