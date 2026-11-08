use super::*;
#[test]
fn picks_the_active_language() {
    set(Lang::En);
    assert_eq!(tr("sim", "yes"), "yes");
    set(Lang::Pt);
    assert_eq!(tr("sim", "yes"), "sim");
    set(Lang::En);
}

#[test]
fn catalogs_are_complete_in_both_languages() {
    assert!(
        keys_without_english().is_empty(),
        "every Portuguese key needs an English entry: {:?}",
        keys_without_english()
    );
    for (k, _) in EN {
        assert!(
            PT.iter().any(|(p, _)| p == k),
            "missing pt-BR entry for {k}"
        );
    }
}

#[test]
fn lookup_falls_back_to_english_then_to_the_key() {
    set(Lang::Pt);
    assert_eq!(t("menu.survival"), "Sobrevivência");
    assert_eq!(t("menu.nonexistent.key"), "menu.nonexistent.key");
    assert!(missing_logged().contains(&"menu.nonexistent.key"));
    set(Lang::En);
    assert_eq!(t("menu.survival"), "Survival");
}

#[test]
fn missing_keys_are_logged_only_once() {
    set(Lang::En);
    let n = missing_logged().len();
    let _ = t("another.missing.key");
    let _ = t("another.missing.key");
    assert_eq!(missing_logged().len(), n + 1);
}

#[test]
fn english_is_the_default_locale() {
    assert_eq!(default_lang(), Lang::En);
}

#[test]
fn migrated_screens_have_no_hardcoded_portuguese() {
    let root = env!("CARGO_MANIFEST_DIR");
    let files = [
        "src/app.rs",
        "src/app/replay_browser.rs",
        "src/app/survival.rs",
    ];
    let phrases = [
        "Revanche",
        "RENOMEAR REPLAY",
        "FIM DE JOGO",
        "Nenhuma luta gravada",
        "Pressione DELETE",
        "Tentar de novo",
        "Monte sua equipe",
        "Escolha o oponente",
        "CO-OP 2P: LIGADO",
        "Personalizar teclado",
        "Menu principal",
        "Reiniciar luta",
        "Trocar personagens",
        "JOGADOR 1: escolha",
        "SOBREVIVÊNCIA · RODADA",
        "{score} PONTOS",
    ];
    for file in files {
        let text = std::fs::read_to_string(format!("{root}/{file}"))
            .unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
        for line in text.lines().filter(|l| !l.trim_start().starts_with("//")) {
            for phrase in phrases {
                assert!(
                    !line.contains(phrase),
                    "{file} still has hardcoded UI text {phrase:?}: {line}"
                );
            }
        }
    }
}

#[test]
fn spanish_catalog_covers_every_key() {
    for (k, _) in EN {
        assert!(ES.iter().any(|(e, _)| e == k), "missing es entry for {k}");
    }
}

#[test]
fn language_codes_round_trip() {
    for (l, _) in LANGS {
        assert_eq!(Lang::from_code(l.code()), l);
    }
    assert_eq!(Lang::En.next(1), Lang::Pt);
    assert_eq!(Lang::En.next(-1), Lang::Es);
}

#[test]
fn spanish_inline_translation_falls_back_to_english() {
    set(Lang::Es);
    assert_eq!(tr("Voltar", "Back"), "Volver");
    assert_eq!(tr("x", "Unmapped text"), "Unmapped text");
    assert_eq!(tr3("a", "b", "c"), "c");
    set(Lang::En);
}

#[test]
fn fill_replaces_named_slots() {
    assert_eq!(fill("{a} and {b}", &[("a", "1"), ("b", "2")]), "1 and 2");
}

#[test]
fn system_locale_codes_map_to_supported_languages() {
    assert_eq!(system::from_code("pt-BR"), Lang::Pt);
    assert_eq!(system::from_code("es_MX.UTF-8"), Lang::Es);
    assert_eq!(system::from_code("de-DE"), Lang::En);
    assert_eq!(system::from_code(""), Lang::En);
}
