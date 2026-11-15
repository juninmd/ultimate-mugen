use super::context::{Context, Value, KNOWN_VARIABLES};
use super::lexer::{tokenize, Token};

#[derive(Clone, Debug)]
enum Node {
    Literal(Value),
    Variable(String),
    Not(Box<Node>),
    Binary(&'static str, Box<Node>, Box<Node>),
}

#[derive(Clone, Debug)]
pub struct Condition {
    root: Node,
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

const LEVELS: [&[&str]; 3] = [&["||"], &["&&"], &["==", "=", "!=", "<=", ">=", "<", ">"]];

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn binary(&mut self, level: usize) -> Result<Node, String> {
        if level == LEVELS.len() {
            return self.unary();
        }
        let mut left = self.binary(level + 1)?;
        while let Some(Token::Op(op)) = self.peek().cloned() {
            if !LEVELS[level].contains(&op) {
                break;
            }
            self.at += 1;
            let right = self.binary(level + 1)?;
            let op = if op == "=" { "==" } else { op };
            left = Node::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Node, String> {
        match self.peek().cloned() {
            Some(Token::Op("!")) => {
                self.at += 1;
                Ok(Node::Not(Box::new(self.unary()?)))
            }
            Some(Token::Open) => {
                self.at += 1;
                let inner = self.binary(0)?;
                match self.peek() {
                    Some(Token::Close) => {
                        self.at += 1;
                        Ok(inner)
                    }
                    _ => Err("missing ')'".into()),
                }
            }
            Some(Token::Number(n)) => {
                self.at += 1;
                Ok(Node::Literal(Value::Int(n)))
            }
            Some(Token::Text(t)) => {
                self.at += 1;
                Ok(Node::Literal(Value::Text(t)))
            }
            Some(Token::Name(name)) => {
                self.at += 1;
                Ok(match name.as_str() {
                    "true" => Node::Literal(Value::Int(1)),
                    "false" => Node::Literal(Value::Int(0)),
                    _ => Node::Variable(name),
                })
            }
            other => Err(format!("unexpected {other:?}")),
        }
    }
}

fn collect_variables(node: &Node, out: &mut Vec<String>) {
    match node {
        Node::Variable(name) => out.push(name.clone()),
        Node::Not(inner) => collect_variables(inner, out),
        Node::Binary(_, a, b) => {
            collect_variables(a, out);
            collect_variables(b, out);
        }
        Node::Literal(_) => {}
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Int(n) => *n != 0,
        Value::Text(t) => !t.is_empty(),
    }
}

fn compare(op: &str, a: &Value, b: &Value) -> Result<bool, String> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(match op {
            "==" => x == y,
            "!=" => x != y,
            "<" => x < y,
            "<=" => x <= y,
            ">" => x > y,
            _ => x >= y,
        }),
        (Value::Text(x), Value::Text(y)) => match op {
            "==" => Ok(x == y),
            "!=" => Ok(x != y),
            _ => Err(format!("'{op}' needs numbers")),
        },
        _ => Err("cannot compare a number with text".into()),
    }
}

fn eval(node: &Node, context: &Context) -> Result<Value, String> {
    match node {
        Node::Literal(v) => Ok(v.clone()),
        Node::Variable(name) => context
            .get(name)
            .cloned()
            .ok_or_else(|| format!("variable '{name}' has no value here")),
        Node::Not(inner) => Ok(Value::Int(!truthy(&eval(inner, context)?) as i64)),
        Node::Binary(op, a, b) => {
            let left = eval(a, context)?;
            if *op == "||" && truthy(&left) {
                return Ok(Value::Int(1));
            }
            if *op == "&&" && !truthy(&left) {
                return Ok(Value::Int(0));
            }
            let right = eval(b, context)?;
            if *op == "||" || *op == "&&" {
                return Ok(Value::Int(truthy(&right) as i64));
            }
            Ok(Value::Int(compare(op, &left, &right)? as i64))
        }
    }
}

impl Condition {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut parser = Parser {
            tokens: tokenize(source)?,
            at: 0,
        };
        let root = parser.binary(0)?;
        if parser.at != parser.tokens.len() {
            return Err("unexpected trailing tokens".into());
        }
        let mut names = Vec::new();
        collect_variables(&root, &mut names);
        if let Some(unknown) = names
            .iter()
            .find(|n| !KNOWN_VARIABLES.contains(&n.as_str()))
        {
            return Err(format!(
                "unknown variable '{unknown}' (known: {})",
                KNOWN_VARIABLES.join(", ")
            ));
        }
        Ok(Self { root })
    }

    pub fn eval(&self, context: &Context) -> Result<bool, String> {
        eval(&self.root, context).map(|v| truthy(&v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context::default()
            .with_int("stats.arcade.clear", 2)
            .with_int("consecutive_wins", 7)
            .with_text("mode", "arcade")
            .with_int("netplay", 0)
    }

    #[test]
    fn evaluates_comparisons_and_logic() {
        let yes = |s: &str| Condition::parse(s).unwrap().eval(&ctx()).unwrap();
        assert!(yes("consecutive_wins == 7"));
        assert!(yes("stats.arcade.clear >= 1 && mode == \"arcade\""));
        assert!(yes("netplay || stats.arcade.clear > 1"));
        assert!(!yes("!(consecutive_wins >= 7)"));
        assert!(yes("mode != \"bossrush\""));
        assert!(yes("true"));
    }

    #[test]
    fn rejects_unknown_variables_and_bad_syntax() {
        assert!(Condition::parse("wins > 1")
            .unwrap_err()
            .contains("unknown variable"));
        assert!(Condition::parse("(1 == 1").is_err());
        assert!(Condition::parse("1 ==").is_err());
        assert!(Condition::parse("1 == 1 2").is_err());
    }

    #[test]
    fn type_mismatch_and_missing_values_are_errors() {
        let c = Condition::parse("mode == 1").unwrap();
        assert!(c.eval(&ctx()).is_err());
        let c = Condition::parse("match_no == 1").unwrap();
        assert!(c.eval(&ctx()).is_err());
    }
}
