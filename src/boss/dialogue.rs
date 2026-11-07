pub struct Page {
    pub who: u8,
    pub text: String,
}

fn pg(who: u8, text: impl Into<String>) -> Page {
    Page {
        who,
        text: text.into(),
    }
}

pub fn script(
    kind: u8,
    hero: &str,
    hero_quote: &str,
    boss_id: &str,
    boss_name: &str,
    boss_line: &str,
) -> Vec<Page> {
    let first = hero.split_whitespace().next().unwrap_or(hero).to_string();
    let clean_boss = boss_id.strip_prefix("umf:").unwrap_or(boss_id);
    match kind {
        0 => vec![
            pg(0, "O Torneio do Eclipse Escarlate reúne os maiores lutadores de todas as eras."),
            pg(0, "Mas diz a lenda que, no fim do caminho, aguarda um adversário que nunca perdeu — e nunca precisou lutar de verdade."),
            pg(1, "Não importa quem me espera no final. Eu vou até o fim."),
        ],
        1 => match clean_boss {
            "garan" => vec![
                pg(0, "Oito rivais caíram. O chão treme e chamas sulfúricas devoram o horizonte."),
                pg(2, if boss_line.is_empty() { "Seu medo tem um gosto... delicioso." } else { boss_line }),
                pg(1, "Um oni milenar... seu reinado de terror termina aqui!"),
                pg(2, format!("{boss_name}. Aquele que transformará sua carne e ossos em cinzas.")),
                pg(1, "Pode vir com tudo. Não tenho medo de fogo."),
                pg(2, "Então queime no abismo infernal!"),
            ],
            "valdric" => vec![
                pg(0, "Oito rivais caíram. A fortaleza imperial de Neo-Metropolis se ergue impenetrável."),
                pg(2, if boss_line.is_empty() { "Um príncipe não luta com plebeus. Hoje abro exceção." } else { boss_line }),
                pg(1, "Valdric Morn... sua sede de tirania acaba neste ringue."),
                pg(2, format!("{boss_name}. Curve-se diante da superioridade absoluta da coroa imperial!")),
                pg(1, "Coroas caem, e a sua não é diferente."),
                pg(2, "Audacioso. Mas a história não perdoa rebeldes fracos!"),
            ],
            _ => vec![
                pg(0, "Oito rivais caíram. O salão final se abre sob uma lua vermelha."),
                pg(2, if boss_line.is_empty() { "Você chegou até aqui... exatamente como eu havia previsto." } else { boss_line }),
                pg(1, "Quem é você?"),
                pg(2, format!("{boss_name}. Aquele que já viu o fim desta luta antes mesmo de ela começar.")),
                pg(1, "Então vai ver você perder também."),
                pg(2, "Que adorável. Mostre-me... o que existe depois do final."),
            ],
        },
        _ => match clean_boss {
            "garan" => vec![
                pg(2, "O portão de cinzas... está se fechando... desgraçado!..."),
                pg(1, if hero_quote.is_empty() { "Volte para as profundezas de onde nunca deveria ter saído.".to_string() } else { hero_quote.to_string() }),
                pg(0, "As chamas vulcânicas se extinguem sob o amanhecer límpido."),
                pg(0, format!("O tormento foi banido — e a lenda de {first} é forjada na vitória.")),
            ],
            "valdric" => vec![
                pg(2, "Como pode um príncipe herdeiro sucumbir diante de tamanha insolência?..."),
                pg(1, if hero_quote.is_empty() { "Seu orgulho cego foi sua ruína, príncipe.".to_string() } else { hero_quote.to_string() }),
                pg(0, "Os canhões imperiais silenciam perante a bravura incontestável."),
                pg(0, format!("O império caiu — e um novo campeão desponta triunfante: {first}.")),
            ],
            _ => vec![
                pg(2, "Impossível... eu havia previsto cada movimento seu..."),
                pg(1, if hero_quote.is_empty() { "Você previu o que eu faria. Não o quanto eu queria vencer.".to_string() } else { hero_quote.to_string() }),
                pg(0, "A lua vermelha se parte em mil pedaços de luz."),
                pg(0, format!("O torneio chegou ao fim — e uma nova lenda nasce: {first}.")),
            ],
        },
    }
}
