use super::*;

impl App {
    pub(crate) fn fit_label(&self, text: &str, size: f32, max_w: f32) -> String {
        if self.ui.measure(text, size, true) <= max_w {
            return text.to_string();
        }
        let mut kept: Vec<char> = text.chars().collect();
        while !kept.is_empty() {
            kept.pop();
            let candidate = format!("{}…", kept.iter().collect::<String>().trim_end());
            if self.ui.measure(&candidate, size, true) <= max_w {
                return candidate;
            }
        }
        "…".to_string()
    }
}
