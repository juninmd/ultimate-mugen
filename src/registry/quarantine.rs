#[derive(Clone, Debug)]
pub struct Quarantined {
    pub name: String,
    pub def: String,
    pub problems: Vec<String>,
}

pub fn reason_label(problem: &str) -> &'static str {
    if problem.starts_with("sprite ") {
        "arquivo de sprites (.sff) ausente"
    } else if problem.starts_with("anim ") {
        "arquivo de animação (.air) ausente"
    } else if problem.starts_with("cns ") {
        "arquivo de estados (.cns) ausente"
    } else if problem.starts_with("[Files] sem") {
        "[Files] sem a chave obrigatória"
    } else if problem.starts_with("sem seção") {
        ".def sem seção [Files]"
    } else {
        "arquivo .def não encontrado"
    }
}

pub fn reason_counts(items: &[Quarantined]) -> Vec<(&'static str, usize)> {
    let mut counts: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for item in items {
        let mut seen: Vec<&'static str> = Vec::new();
        for p in &item.problems {
            let label = reason_label(p);
            if !seen.contains(&label) {
                seen.push(label);
                *counts.entry(label).or_default() += 1;
            }
        }
    }
    let mut out: Vec<_> = counts.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_each_reason_once_per_character() {
        let items = vec![
            Quarantined {
                name: "a".into(),
                def: "a.def".into(),
                problems: vec![
                    "anim 'a.air' não existe".into(),
                    "cns 'a.cns' não existe".into(),
                ],
            },
            Quarantined {
                name: "b".into(),
                def: "b.def".into(),
                problems: vec!["anim 'b.air' não existe".into()],
            },
        ];
        let counts = reason_counts(&items);
        assert_eq!(counts[0], ("arquivo de animação (.air) ausente", 2));
        assert_eq!(counts[1], ("arquivo de estados (.cns) ausente", 1));
    }
}
