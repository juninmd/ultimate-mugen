use crate::i18n::tr3;

pub(crate) fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

pub(crate) fn by_author(author: &str, max: usize) -> Option<String> {
    let author = author.trim();
    if author.is_empty() {
        return None;
    }
    let label = tr3("por", "by", "por");
    Some(truncate(&format!("{label} {author}"), max))
}

pub(crate) fn series_line(series: &str, author: &str, max: usize) -> String {
    match by_author(author, max) {
        None => truncate(series, max),
        Some(by) if series.is_empty() => by,
        Some(by) => truncate(&format!("{series} · {by}"), max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_author_is_omitted() {
        assert_eq!(by_author("  ", 20), None);
        assert_eq!(series_line("Saga", "", 20), "Saga");
    }

    #[test]
    fn long_text_is_truncated_with_ellipsis() {
        let s = truncate("abcdefghijklmnop", 8);
        assert_eq!(s.chars().count(), 8);
        assert!(s.ends_with('…'));
    }
}

impl crate::app::App {
    pub(crate) fn select_author(&self, author: &str, cx: f32, y: f32) {
        if let Some(by) = by_author(author, 30) {
            self.ui.text(&by, cx, y, 13.0, crate::ui::hex("#22e8ff"), crate::ui::Align::Center, false);
        }
    }
}
