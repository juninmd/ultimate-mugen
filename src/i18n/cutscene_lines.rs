use super::{get, Lang};
use std::sync::OnceLock;

fn table() -> &'static [[String; 3]] {
    static TABLE: OnceLock<Vec<[String; 3]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/i18n/cutscene_lines.json"))
            .unwrap_or_default()
    })
}

pub fn shown(text: &str) -> &str {
    let column = match get() {
        Lang::Pt => 0,
        Lang::En => 1,
        Lang::Es => 2,
    };
    table()
        .iter()
        .find(|row| row[0] == text)
        .map_or(text, |row| row[column].as_str())
}

pub fn fill(template: &str, hero: &str, boss: &str) -> String {
    shown(template)
        .replace("{hero}", hero)
        .replace("{boss}", boss)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quoted(line: &str) -> Vec<&str> {
        line.split('"')
            .skip(1)
            .step_by(2)
            .filter(|s| s.chars().count() >= 12)
            .collect()
    }

    #[test]
    fn table_parses_with_matching_placeholders() {
        assert!(table().len() > 190);
        for row in table() {
            assert!(row.iter().all(|cell| !cell.is_empty()));
            for tag in ["{hero}", "{boss}"] {
                assert!(
                    row.iter()
                        .all(|cell| cell.contains(tag) == row[0].contains(tag)),
                    "{tag} in {}",
                    row[0]
                );
            }
        }
    }

    #[test]
    fn every_cutscene_line_is_translated() {
        let dir = env!("CARGO_MANIFEST_DIR");
        for name in ["app/cutscene/story.rs", "app/cutscene/climax.rs", "app/cutscene/endings.rs", "boss/dialogue.rs"] {
            let source = std::fs::read_to_string(format!("{dir}/src/{name}")).unwrap();
            for line in source
                .lines()
                .filter(|l| !l.contains("tr3") && !l.contains("contains("))
            {
                for text in quoted(line) {
                    assert!(
                        table().iter().any(|row| row[0] == text),
                        "{name}: no translation for {text}"
                    );
                }
            }
        }
    }

    #[test]
    fn fill_replaces_names_and_unknown_text_passes_through() {
        assert_eq!(shown("M.U.G.E.N"), "M.U.G.E.N");
        assert_eq!(fill("{hero} x {boss}", "A", "B"), "A x B");
    }
}
