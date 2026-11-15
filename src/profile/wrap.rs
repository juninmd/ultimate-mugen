pub(crate) fn wrap(text: &str, max_width: f32, measure: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let start = line.len();
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
        if start > 0 && measure(&line) > max_width {
            line.truncate(start);
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
