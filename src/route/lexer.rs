#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Number(i64),
    Text(String),
    Name(String),
    Op(&'static str),
    Open,
    Close,
}

const OPERATORS: [&str; 10] = ["||", "&&", "==", "!=", "<=", ">=", "<", ">", "!", "="];

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '(' {
            tokens.push(Token::Open);
            i += 1;
        } else if c == ')' {
            tokens.push(Token::Close);
            i += 1;
        } else if c == '"' {
            let end = chars[i + 1..]
                .iter()
                .position(|&x| x == '"')
                .ok_or("unterminated string")?;
            tokens.push(Token::Text(chars[i + 1..i + 1 + end].iter().collect()));
            i += end + 2;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            tokens.push(Token::Number(text.parse().map_err(|_| "number too large")?));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
            tokens.push(Token::Name(chars[start..i].iter().collect()));
        } else {
            let rest: String = chars[i..].iter().collect();
            let op = OPERATORS
                .iter()
                .find(|op| rest.starts_with(**op))
                .ok_or_else(|| format!("unexpected character '{c}'"))?;
            tokens.push(Token::Op(op));
            i += op.len();
        }
    }
    Ok(tokens)
}
