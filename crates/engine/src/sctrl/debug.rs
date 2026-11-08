use crate::chardef::Controller;
use crate::matchs::Match;

pub fn display(m: &mut Match, i: usize, sc: &Controller, append: bool) {
    let template = sc
        .raw("text")
        .map(ume_formats::ini::unquote)
        .unwrap_or_default();
    let args: Vec<f64> = sc
        .param("params")
        .map(|p| {
            (0..p.exprs.len())
                .map(|k| m.num(i, sc, "params", 0.0, k))
                .collect()
        })
        .unwrap_or_default();
    let text = format_clipboard(&template, &args);
    let f = &mut m.fighters[i];
    if !append {
        f.clipboard.clear();
    }
    for (k, line) in text.split('\n').enumerate() {
        if k == 0 && append {
            if let Some(last) = f.clipboard.last_mut() {
                last.push_str(line);
                continue;
            }
        }
        f.clipboard.push(line.to_string());
    }
    let excess = f.clipboard.len().saturating_sub(10);
    f.clipboard.drain(..excess);
}

pub fn format_clipboard(template: &str, args: &[f64]) -> String {
    let mut out = String::new();
    let mut it = template.chars().peekable();
    let mut next = args.iter();
    while let Some(c) = it.next() {
        match (c, it.peek().copied()) {
            ('%', Some('d' | 'i')) => {
                it.next();
                out += &next
                    .next()
                    .map(|v| (*v as i64).to_string())
                    .unwrap_or_default();
            }
            ('%', Some('f')) => {
                it.next();
                out += &next.next().map(|v| format!("{v:.6}")).unwrap_or_default();
            }
            ('%', Some('%')) => {
                it.next();
                out.push('%');
            }
            ('\\', Some('n')) => {
                it.next();
                out.push('\n');
            }
            ('\\', Some('t')) => {
                it.next();
                out.push('\t');
            }
            _ => out.push(c),
        }
    }
    out
}
