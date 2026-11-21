use super::builders::{floor_roll, flyby, ribbon};
use super::{ActiveEgg, EggSprite};
use crate::i18n::tr3;

pub(crate) fn dojo_crane() -> ActiveEgg {
    flyby(
        EggSprite::OrigamiCrane,
        tr3("GROU DE ORIGAMI!", "MASTER'S PAPER CRANE!", "¡GRULLA DE ORIGAMI!"),
        tr3("BÊNÇÃO DO TELHADO DO DOJO", "BLESSING OF THE SUNSET DOJO", "BENDICIÓN DEL DOJO AL ATARDECER"),
        [1.0, 0.78, 0.42],
        false,
    )
}

pub(crate) fn dojo_roof_rumble() -> ActiveEgg {
    ribbon(
        EggSprite::OrigamiCrane,
        tr3("TELHAS TREMULAS!", "ROOFTOP TREMOR!", "¡TEJAS TEMBLANDO!"),
        tr3("O VELHO DOJO SENTIU ESSE GOLPE!", "THE ANCIENT ROOF SHOOK!", "¡EL VIEJO DOJO TEMBLÓ!"),
        [1.0, 0.56, 0.32],
        0,
    )
}

pub(crate) fn neon_ufo() -> ActiveEgg {
    flyby(
        EggSprite::UfoNeon,
        tr3("OVNI EM NEO-METRÓPOLE!", "CYBER UFO SIGHTING!", "¡OVNI EN NEO-METRÓPOLIS!"),
        tr3("VISITANTE DE 2099 DETECTADO", "UNIDENTIFIED 2099 FLYBY", "VISITANTE DE 2099 DETECTADO"),
        [0.32, 0.94, 1.0],
        true,
    )
}

pub(crate) fn neon_overload() -> ActiveEgg {
    ribbon(
        EggSprite::UfoNeon,
        tr3("SOBRECARGA // 2099!", "SYSTEM OVERLOAD // 2099!", "¡SOBRECARGA // 2099!"),
        tr3("REDE NEON EM CURTO-CIRCUITO", "SKYLINE GRID SURGING", "RED NEÓN EN CORTOCIRCUITO"),
        [1.0, 0.24, 0.82],
        0,
    )
}

pub(crate) fn bamboo_spirits() -> ActiveEgg {
    flyby(
        EggSprite::KitsuneMask,
        tr3("ESPÍRITOS DO BAMBUZAL!", "MOONLIT BAMBOO SPIRITS!", "¡ESPÍRITUS DEL BAMBÚ!"),
        tr3("A FLORESTA ASSISTE AO DUELO", "THE GROVE WATCHES IN SILENCE", "EL BOSQUE OBSERVA EL DUELO"),
        [0.42, 0.96, 0.78],
        false,
    )
}

pub(crate) fn plains_galleon() -> ActiveEgg {
    flyby(
        EggSprite::OrigamiCrane,
        tr3("EL VIENTO SALADO!", "SKY GALLEON SIGHTING!", "¡EL VIENTO SALADO!"),
        tr3("O NAVIO DE MARLOW CRUZA AS NUVENS", "MARLOW'S SHIP CROSSES THE CLOUDS", "EL GALEÓN DE MARLOW EN LAS NUBES"),
        [0.52, 0.84, 1.0],
        true,
    )
}

pub(crate) fn spirit_kitsune() -> ActiveEgg {
    flyby(
        EggSprite::KitsuneMask,
        tr3("APARIÇÃO KITSUNE!", "KITSUNE APPARITION!", "¡APARICIÓN KITSUNE!"),
        tr3("O PORTÃO DOS ESPÍRITOS DESPERTOU", "THE TORII GATE AWAKENS", "EL PORTAL ESPIRITUAL DESPIERTA"),
        [1.0, 0.34, 0.44],
        false,
    )
}

pub(crate) fn volcano_geyser() -> ActiveEgg {
    ribbon(
        EggSprite::ScarabLaurel,
        tr3("ERUPÇÃO SÍSMICA!", "MAGMA GEYSER SPIKE!", "¡ERUPCIÓN SÍSMICA!"),
        tr3("A CRATERA RUGE COM O IMPACTO!", "THE CALDERA ROARS BACK!", "¡EL CRÁTER RUGE CON EL IMPACTO!"),
        [1.0, 0.36, 0.16],
        0,
    )
}

pub(crate) fn brazil_capybara() -> ActiveEgg {
    floor_roll(
        EggSprite::Capybara,
        tr3("CAPIVARA DE BOA!", "CHILL CAPYBARA CAMEO!", "¡CAPIBARA TRANQUILO!"),
        tr3("COM TANGERINA NA CABEÇA NA PRAÇA", "VIBING WITH A TANGERINE ON HEAD", "CON MANDARINA EN LA CABEZA"),
        [0.98, 0.68, 0.28],
        false,
    )
}

pub(crate) fn brazil_caramelo() -> ActiveEgg {
    ribbon(
        EggSprite::Capybara,
        tr3("CARAMELO NA ÁREA!", "CARAMELO DOG ON PATROL!", "¡PERRO CARAMELO EN LA PLAZA!"),
        tr3("PATRIMÔNIO OFICIAL DA PRAÇA!", "OFFICIAL PLAZA GUARDIAN!", "¡GUARDIÁN OFICIAL DE LA PLAZA!"),
        [0.96, 0.72, 0.32],
        0,
    )
}

pub(crate) fn christ_toucan() -> ActiveEgg {
    flyby(
        EggSprite::Toucan,
        tr3("TUCANO NO CORCOVADO!", "CORCOVADO TOUCAN FLYBY!", "¡TUCÁN EN EL CORCOVADO!"),
        tr3("VOO TROPICAL SOBRE O RIO", "TROPICAL GLIDE OVER RIO", "VUELO TROPICAL SOBRE RÍO"),
        [1.0, 0.82, 0.22],
        false,
    )
}

pub(crate) fn great_wall_kite() -> ActiveEgg {
    flyby(
        EggSprite::OrigamiCrane,
        tr3("DRAGÃO DE PAPEL!", "SKY DRAGON KITE!", "¡COMETA DRAGÓN!"),
        tr3("DANÇANDO SOBRE A GRANDE MURALHA", "SOARING OVER THE GREAT WALL", "VOLANDO SOBRE LA GRAN MURALLA"),
        [1.0, 0.74, 0.24],
        true,
    )
}

pub(crate) fn petra_scarab() -> ActiveEgg {
    flyby(
        EggSprite::ScarabLaurel,
        tr3("ESCARAVELHO DE PETRA!", "ROSE CANYON SCARAB!", "¡ESCARABAJO DE PETRA!"),
        tr3("MIRAGEM DOURADA EM AL-KHAZNEH", "GOLDEN MIRAGE AT AL-KHAZNEH", "ESPEJISMO DORADO EN AL-KHAZNEH"),
        [1.0, 0.64, 0.36],
        false,
    )
}

pub(crate) fn texas_noon() -> ActiveEgg {
    floor_roll(
        EggSprite::Tumbleweed,
        tr3("MEIO-DIA NO TEXAS!", "HIGH NOON IN TEXAS!", "¡MEDIODÍA EN TEXAS!"),
        tr3("VENTO SECO NA RUA PRINCIPAL", "DUSTY WIND DOWN MAIN STREET", "VIENTO SECO EN LA CALLE PRINCIPAL"),
        [0.94, 0.68, 0.34],
        true,
    )
}

pub(crate) fn chichen_serpent() -> ActiveEgg {
    ribbon(
        EggSprite::ScarabLaurel,
        tr3("SOMBRA DE KUKULCÁN!", "FEATHERED SERPENT SHADOW!", "¡SOMBRA DE KUKULCÁN!"),
        tr3("A PIRÂMIDE BRILHA EM ESMERALDA", "THE PYRAMID GLOWS EMERALD", "LA PIRÁMIDE BRILLA EN ESMERALDA"),
        [0.24, 0.92, 0.64],
        0,
    )
}

pub(crate) fn colosseum_laurel() -> ActiveEgg {
    ribbon(
        EggSprite::ScarabLaurel,
        tr3("AVE GLADIATOR!", "AVE GLADIATOR!", "¡AVE GLADIATOR!"),
        tr3("O IMPERADOR APROVA O COMBATE!", "THE EMPEROR GIVES THUMBS UP!", "¡EL EMPERADOR APRUEBA EL DUELO!"),
        [1.0, 0.84, 0.32],
        0,
    )
}

pub(crate) fn taj_doves() -> ActiveEgg {
    flyby(
        EggSprite::OrigamiCrane,
        tr3("POMBAS DE AGRA!", "WHITE DOVES OF AGRA!", "¡PALOMAS DE AGRA!"),
        tr3("VOANDO SOBRE O PALÁCIO DE MÁRMORE", "TAKING FLIGHT OVER THE MARBLE", "VOLANDO SOBRE EL MÁRMOL"),
        [0.96, 0.94, 1.0],
        false,
    )
}

pub(crate) fn giza_horus() -> ActiveEgg {
    flyby(
        EggSprite::ScarabLaurel,
        tr3("OLHO DE HÓRUS!", "PHARAOH'S EYE GLINT!", "¡OJO DE HORUS!"),
        tr3("O PODER DE GIZÉ DESPERTA", "ANCIENT GIZA POWER AWAKENS", "EL PODER DE GIZA DESPIERTA"),
        [1.0, 0.82, 0.24],
        true,
    )
}

pub(crate) fn babylon_peacock() -> ActiveEgg {
    flyby(
        EggSprite::Toucan,
        tr3("PLUMAS DA BABILÔNIA!", "HANGING GARDENS PEACOCK!", "¡PLUMAS DE BABILONIA!"),
        tr3("BRISA REAL PELOS JARDINS SUSPENSOS", "ROYAL CANOPY BREEZE", "BRISA REAL EN LOS JARDINES"),
        [0.28, 0.94, 0.72],
        false,
    )
}

pub(crate) fn ancient_wonder(title: &str, sub: &str, color: [f32; 3]) -> ActiveEgg {
    ribbon(EggSprite::ScarabLaurel, title, sub, color, 0)
}
