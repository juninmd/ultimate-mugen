use super::{get, Lang};
use std::sync::OnceLock;

fn table() -> &'static [[String; 3]] {
    static TABLE: OnceLock<Vec<[String; 3]>> = OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(include_str!("../../assets/i18n/fighter_lines.json")).unwrap_or_default())
}

pub fn shown(text: &str) -> &str {
    let column = match get() {
        Lang::Pt => 0,
        Lang::En => 1,
        Lang::Es => 2,
    };
    table()
        .iter()
        .find(|row| row[0] == text || row[1] == text)
        .map_or(text, |row| row[column].as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_parses_with_three_filled_columns() {
        assert!(table().len() > 100);
        assert!(table().iter().all(|row| row.iter().all(|cell| !cell.is_empty())));
    }

    #[test]
    fn every_native_fighter_line_is_translated() {
        for file in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/chars")).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_none_or(|e| e != "json") || path.ends_with("index.json") {
                continue;
            }
            let fighter: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let mut lines: Vec<&str> = ["series", "winQuote", "intro", "taunt"].iter().filter_map(|k| fighter[*k].as_str()).collect();
            if let Some(rivals) = fighter["rivals"].as_object() {
                lines.extend(rivals.values().flat_map(|r| ["intro", "taunt"].map(|k| r[k].as_str())).flatten());
            }
            for line in lines {
                assert!(table().iter().any(|row| row[0] == line || row[1] == line), "{}: no translation for {line}", path.display());
            }
        }
    }

    #[test]
    fn unknown_text_passes_through() {
        assert_eq!(shown("M.U.G.E.N"), "M.U.G.E.N");
    }
}
