use super::*;
use crate::expr::{parse, Expr};

struct H;
impl Host for H {
    fn trigger(&self, name: &str, args: &[Node], comp: Option<char>, _: Option<&str>) -> f64 {
        let a: f64 = args.iter().map(|n| eval(n, self)).sum();
        let k = name
            .bytes()
            .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32))
            % 23;
        k as f64 - 6.0 + a * 0.5 + if comp == Some('y') { -1.25 } else { 0.0 }
    }
    fn redirect(&self, _: &str, _: Option<f64>) -> Option<Self> {
        Some(H)
    }
    fn cmpstr(&self, name: &str, neg: bool, v: &[String]) -> bool {
        (name.len() + v.len()).is_multiple_of(2) != neg
    }
    fn special(&self, _: &str, op: &str, a: f64, _: Option<&str>, _: Option<f64>) -> bool {
        (a as i64 % 2 == 0) == (op == "=")
    }
    fn assign(&self, _: &str, _: i32, v: f64) -> f64 {
        v
    }
}

fn same(src: &str) {
    let n = parse(src).unwrap();
    let tree = eval(&n, &H);
    let code = Expr::new(n).eval(&H);
    assert!(
        tree == code || (tree.is_nan() && code.is_nan()),
        "{src}: tree {tree} != code {code}"
    );
}

#[test]
fn matches_tree_on_samples() {
    for s in [
        "1 + 2 * 3",
        "7/2",
        "7.0/2",
        "-7/2",
        "2**3",
        "2.0**0.5",
        "7%3",
        "time = [5, 7]",
        "time != (7, 9]",
        "!time || 0",
        "time && var(1)",
        "ifelse(time > 3, 10, 20)",
        "cond(0, 1, 2.5)",
        "vel y < 0 && pos x",
        "~5",
        "3 ^^ 0",
        "6 & 3 | 8 ^ 1",
        "var(3) := 4 + 1",
        "p2, life > 0",
        "statetype = S",
        "animelem = 3, >= 2",
        "1/0",
        "5 % 0",
        "-(2 + 3) * 2.5",
    ] {
        same(s);
    }
}

#[test]
fn matches_tree_on_common1() {
    let src = include_str!("../common1.cns");
    let mut n = 0;
    for line in src.lines() {
        let l = line.split(';').next().unwrap_or("");
        let Some((k, v)) = l.split_once('=') else {
            continue;
        };
        let k = k.trim().to_ascii_lowercase();
        if !(k.starts_with("trigger") || k == "value" || k == "x" || k == "y") {
            continue;
        }
        if parse(v.trim()).is_ok() {
            same(v.trim());
            n += 1;
        }
    }
    assert!(n > 100, "only {n} expressions");
}

struct Bottom;
impl Host for Bottom {
    fn trigger(&self, name: &str, _: &[Node], _: Option<char>, _: Option<&str>) -> f64 {
        if name == "time" {
            f64::NAN
        } else {
            1.0
        }
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

#[test]
fn propagates_bottom() {
    let run = |s: &str| Expr::new(parse(s).unwrap()).eval(&Bottom);
    assert!(run("time").is_nan());
    assert!(run("time = 0").is_nan());
    assert!(run("time != [1, 5]").is_nan());
    assert!(run("!time").is_nan());
    assert!(run("-time + 1").is_nan());
    assert_eq!(run("time && 1"), 0.0);
    assert_eq!(run("time || 1"), 1.0);
    assert_eq!(run("1 || time"), 1.0);
    assert!(!truthy(run("time")));
}

#[test]
fn folds_constants() {
    assert_eq!(
        Code::compile(&parse("2 * (3 + 4) - 1").unwrap()).constant(),
        Some(13.0)
    );
    assert_eq!(
        Code::compile(&parse("ifelse(1, 5, time)").unwrap()).constant(),
        Some(5.0)
    );
    assert_eq!(
        Code::compile(&parse("10 = [5, 15]").unwrap()).constant(),
        Some(1.0)
    );
    assert!(Code::compile(&parse("time + 1").unwrap())
        .constant()
        .is_none());
}