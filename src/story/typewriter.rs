pub fn reveal(lines: &[String], shown: usize) -> Vec<&str> {
    let mut left = shown;
    lines
        .iter()
        .map(|line| {
            let count = line.chars().count();
            if left >= count {
                left -= count;
                return line.as_str();
            }
            let end = line.char_indices().nth(left).map_or(line.len(), |(at, _)| at);
            left = 0;
            &line[..end]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<String> {
        vec!["hello world".into(), "again".into(), "añb".into()]
    }

    #[test]
    fn nothing_shown_yields_empty_rows_and_everything_shown_yields_the_text() {
        assert!(reveal(&lines(), 0).iter().all(|l| l.is_empty()));
        assert_eq!(reveal(&lines(), 999), ["hello world", "again", "añb"]);
    }

    #[test]
    fn a_partial_word_stays_on_its_final_line() {
        assert_eq!(reveal(&lines(), 13), ["hello world", "ag", ""]);
    }

    #[test]
    fn multibyte_characters_are_cut_on_boundaries() {
        assert_eq!(reveal(&lines(), 11 + 5 + 2), ["hello world", "again", "añ"]);
    }
}

#[cfg(test)]
mod page_tests {
    use super::*;
    use crate::profile::wrap;
    use crate::story::epilogue::merged;

    fn measure(line: &str) -> f32 {
        line.chars().count() as f32 * 11.0
    }

    #[test]
    fn every_typed_frame_of_every_page_stays_inside_its_final_layout() {
        let mut jumped_before = 0;
        for epilogue in merged(None).values() {
            for beat in epilogue.beats("en") {
                let full = wrap(&beat.text, 1040.0, measure);
                for shown in 0..=beat.text.chars().count() {
                    let rows = reveal(&full, shown);
                    assert!(rows.iter().zip(&full).all(|(part, whole)| whole.starts_with(part)));
                    let typed: String = beat.text.chars().take(shown).collect();
                    let old = wrap(&typed, 1040.0, measure);
                    if old.iter().zip(&full).any(|(part, whole)| !whole.starts_with(part.as_str())) {
                        jumped_before += 1;
                    }
                }
            }
        }
        println!("frames where the old prefix wrap disagreed with the final layout: {jumped_before}");
    }
}
