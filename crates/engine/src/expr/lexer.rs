#[derive(Clone, Debug, PartialEq)]
pub(super) enum Tok { Num(String, bool), Id(String), Str(String), Op(&'static str), Eof }

const OPS: [&str; 26] = [":=", "**", "!=", "<=", ">=", "&&", "||", "^^", "=", "<", ">", "+", "-", "*", "/", "%", "!", "~", "&", "|", "^", "(", ")", "[", "]", ","];

pub(super) fn tokenize(src: &str) -> Vec<Tok> {
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