/// "true_z_license_free_chara" -> "True Z", "Thunder_horse_GGX" -> "Thunder Horse GGX"
pub fn pretty_name(n: &str) -> String {
    let mut s = n.replace(['_', '-'], " ");
    for junk in [
        "license free mugen chara for official",
        "license free chara",
        "license free",
        "for official",
    ] {
        if let Some(i) = s.to_ascii_lowercase().find(junk) {
            s.replace_range(i..i + junk.len(), "");
        }
    }
    let words: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            if w.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && w.len() > 1
            {
                w.to_string()
            } else {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            }
        })
        .collect();
    if words.is_empty() {
        n.to_string()
    } else {
        words.join(" ")
    }
}
