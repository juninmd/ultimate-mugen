use super::{get, Lang};

const NAMES: [(&str, &str, &str); 22] = [
    ("Telhado do Dojo — Pôr do Sol", "Dojo Rooftop — Sunset", "Tejado del Dojo — Atardecer"),
    ("Neo-Metrópole 2099", "Neo-Metropolis 2099", "Neo-Metrópolis 2099"),
    ("Bambuzal ao Luar", "Moonlit Bamboo Grove", "Bambú a la Luz de la Luna"),
    ("Planalto Rochoso", "Rocky Plateau", "Meseta Rocosa"),
    ("Portão dos Espíritos", "Spirit Gate", "Puerta de los Espíritus"),
    ("Cratera Vulcânica", "Volcanic Crater", "Cráter Volcánico"),
    ("Praça do Interior — Brasil", "Country Town Square — Brazil", "Plaza del Pueblo — Brasil"),
    ("Grande Muralha da China", "Great Wall of China", "Gran Muralla China"),
    ("Petra — Al-Khazneh", "Petra — Al-Khazneh", "Petra — Al-Khazneh"),
    ("Cristo Redentor — Corcovado", "Christ the Redeemer — Corcovado", "Cristo Redentor — Corcovado"),
    ("Cidade do Interior — Texas, EUA", "Small Town — Texas, USA", "Pueblo del Interior — Texas, EE. UU."),
    ("Chichén Itzá — Templo de Kukulcán", "Chichén Itzá — Temple of Kukulcán", "Chichén Itzá — Templo de Kukulcán"),
    ("Coliseu de Roma", "Colosseum of Rome", "Coliseo de Roma"),
    ("Taj Mahal — Palácio de Mármore", "Taj Mahal — Marble Palace", "Taj Mahal — Palacio de Mármol"),
    ("Pirâmides de Gizé", "Pyramids of Giza", "Pirámides de Guiza"),
    ("Jardins Suspensos da Babilônia", "Hanging Gardens of Babylon", "Jardines Colgantes de Babilonia"),
    ("Templo de Ártemis em Éfeso", "Temple of Artemis at Ephesus", "Templo de Artemisa en Éfeso"),
    ("Estátua de Zeus em Olímpia", "Statue of Zeus at Olympia", "Estatua de Zeus en Olimpia"),
    ("Mausoléu de Halicarnasso", "Mausoleum at Halicarnassus", "Mausoleo de Halicarnaso"),
    ("Colosso de Rodes", "Colossus of Rhodes", "Coloso de Rodas"),
    ("Aeroporto 3D — Pôr do Sol", "3D Airport — Sunset", "Aeropuerto 3D — Atardecer"),
    ("Farol de Alexandria — Pharos", "Lighthouse of Alexandria — Pharos", "Faro de Alejandría — Pharos"),
];

pub fn shown(name: &str) -> &str {
    let lang = get();
    match NAMES.iter().find(|(pt, _, _)| *pt == name) {
        Some((_, en, _)) if lang == Lang::En => en,
        Some((_, _, es)) if lang == Lang::Es => es,
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_stage_name_has_a_translation() {
        let builtin = crate::stages::builtin();
        for stage in builtin.iter().filter(|s| !s.id.starts_with("mugen")) {
            assert!(NAMES.iter().any(|(pt, _, _)| *pt == stage.name), "missing translation for {}", stage.name);
        }
    }

    #[test]
    fn unknown_names_pass_through() {
        assert_eq!(shown("Training Room"), "Training Room");
    }
}
