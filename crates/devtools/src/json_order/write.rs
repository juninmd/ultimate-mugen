use super::value::Json;

#[derive(Clone, Copy)]
pub enum Style {
    Compact,
    Indent(usize),
}

fn escape(text: &str, ascii: bool, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if ascii && !c.is_ascii() => {
                let mut units = [0u16; 2];
                c.encode_utf16(&mut units).iter().for_each(|u| out.push_str(&format!("\\u{u:04x}")));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write(value: &Json, style: Style, ascii: bool, depth: usize, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Num(raw) => out.push_str(raw),
        Json::Str(text) => escape(text, ascii, out),
        Json::Arr(items) if items.is_empty() => out.push_str("[]"),
        Json::Obj(items) if items.is_empty() => out.push_str("{}"),
        Json::Arr(items) => container(out, style, ascii, depth, '[', ']', items.iter().map(|v| (None, v))),
        Json::Obj(items) => container(out, style, ascii, depth, '{', '}', items.iter().map(|(k, v)| (Some(k.as_str()), v))),
    }
}

fn container<'a>(out: &mut String, style: Style, ascii: bool, depth: usize, open: char, close: char, items: impl Iterator<Item = (Option<&'a str>, &'a Json)>) {
    out.push(open);
    let items: Vec<_> = items.collect();
    for (index, (key, value)) in items.iter().enumerate() {
        match style {
            Style::Compact => {
                if index > 0 {
                    out.push_str(", ");
                }
            }
            Style::Indent(width) => {
                if index > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.push_str(&" ".repeat(width * (depth + 1)));
            }
        }
        if let Some(key) = key {
            escape(key, ascii, out);
            out.push_str(": ");
        }
        write(value, style, ascii, depth + 1, out);
    }
    if let Style::Indent(width) = style {
        out.push('\n');
        out.push_str(&" ".repeat(width * depth));
    }
    out.push(close);
}

pub fn to_string(value: &Json, style: Style) -> String {
    let mut out = String::new();
    write(value, style, false, 0, &mut out);
    out
}

pub fn to_ascii_string(value: &Json, style: Style) -> String {
    let mut out = String::new();
    write(value, style, true, 0, &mut out);
    out
}
