use super::builders::{flyby, ribbon, speech};
use super::{ActiveEgg, EggSprite};
use crate::i18n::tr3;

pub(crate) fn mirror_match(name: &str) -> ActiveEgg {
    speech(
        name,
        tr3("Espera... quem é o original aqui?!", "Wait... which one of us is the real one?!", "Espera... ¿quién es el original aquí?!"),
        [0.44, 0.92, 1.0],
        0,
    )
}

pub(crate) fn suzume_oboro() -> ActiveEgg {
    ribbon(
        EggSprite::EclipseMoon,
        tr3("PELO CLÃ KAGEROU!", "FOR THE KAGEROU CLAN!", "¡POR EL CLAN KAGEROU!"),
        tr3("SUZUME CORTA O FIO DO ECLIPSE", "SUZUME HUNTS OBORO'S THREAD", "SUZUME CORTA EL HILO DEL ECLIPSE"),
        [0.98, 0.24, 0.42],
        0,
    )
}

pub(crate) fn sylene_garan() -> ActiveEgg {
    ribbon(
        EggSprite::EclipseMoon,
        tr3("BOREALIA NÃO ESQUECE!", "BOREALIA REMEMBERS!", "¡BOREALIA NO OLVIDA!"),
        tr3("GELO ÁRTICO CONTRA CINZAS VIOLETAS", "ARCTIC ICE VS ASHEN FLAMES", "HIELO ÁRTICO CONTRA LLAMAS VIOLETAS"),
        [0.46, 0.84, 1.0],
        0,
    )
}

pub(crate) fn kaito_oboro() -> ActiveEgg {
    ribbon(
        EggSprite::EclipseMoon,
        tr3("O VENTO DISSIPA O ECLIPSE!", "THE WIND CLEARS THE ECLIPSE!", "¡EL VIENTO DISIPA EL ECLIPSE!"),
        tr3("TEMPESTADE DE IZU CONTRA LUA ESCARLATE", "STORM OF IZU VS SCARLET ECLIPSE", "TORMENTA DE IZU VS ECLIPSE ESCARLATA"),
        [0.36, 0.88, 1.0],
        0,
    )
}

pub(crate) fn bruno_home_crowd(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("A TORCIDA É SUA, BRUNO!", "HOME CROWD FOR BRUNO!", "¡LA AFICIÓN ES TUYA, BRUNO!"),
        tr3("O TRICAMPEÃO DE MINAS EM CASA!", "MINAS GERAIS CHAMPION AT HOME!", "¡EL TRICAMPEÓN EN CASA!"),
        [0.24, 0.9, 0.44],
        side,
    )
}

pub(crate) fn volta_neon(side: usize) -> ActiveEgg {
    speech(
        "VOLTA",
        tr3("Firmware: LIVRE-ARBÍTRIO // Bem-vindo de volta!", "Firmware: FREE WILL // Back at the lab!", "Firmware: LIBRE ALBEDRÍO // ¡De vuelta!"),
        [0.28, 0.94, 1.0],
        side,
    )
}

pub(crate) fn akio_fired_up(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::RamenBowl,
        tr3("AGORA SIM FICOU BOM!", "NOW I'M FIRED UP!", "¡AHORA SÍ SE PUSO BUENO!"),
        tr3("ESPÍRITO DO DOJO QUEDA ESTELAR!", "STARFALL DOJO NEVER QUITS!", "¡EL DOJO ESTELAR NO SE RINDE!"),
        [1.0, 0.54, 0.2],
        side,
    )
}

pub(crate) fn haruki_apology(side: usize) -> ActiveEgg {
    speech(
        "HARUKI",
        tr3("Desculpa!! Você tá bem?! Foi o titã!", "Sorry!! Are you okay?! The titan slipped!", "¡¡Perdón!! ¿Estás bien?! ¡Fue el titán!"),
        [0.68, 0.84, 1.0],
        side,
    )
}

pub(crate) fn marlow_valdric(side: usize) -> ActiveEgg {
    speech(
        "MARLOW",
        tr3("Bela coroa imperial... posso pegar emprestado?", "Nice imperial crown... mind if I borrow it?", "Linda corona imperial... ¿me la prestas?"),
        [1.0, 0.78, 0.24],
        side,
    )
}

pub(crate) fn kaito_jin() -> ActiveEgg {
    ribbon(
        EggSprite::OrigamiCrane,
        tr3("CORRIDA CONTRA O VENTO!", "RACE AGAINST THE WIND!", "¡CARRERA CONTRA EL VIENTO!"),
        tr3("PENHASCOS DE IZU VS VALE DE KISO", "IZU CLIFFS VS WIND VALLEY", "ACANTILADOS DE IZU VS VALLE DEL VIENTO"),
        [0.42, 0.96, 0.76],
        0,
    )
}

pub(crate) fn shion_kaname() -> ActiveEgg {
    ribbon(
        EggSprite::CrossedBlades,
        tr3("AÇO CONTRA JULGAMENTO!", "STEEL MEETS JUDGMENT!", "¡ACERO CONTRA JUICIO!"),
        tr3("LÂMINA DE METEORITO VS KATANA ESPIRITUAL", "METEORITE BLADE VS SPIRIT KATANA", "HOJA DE METEORITO VS KATANA ESPIRITUAL"),
        [0.86, 0.92, 1.0],
        0,
    )
}

pub(crate) fn lian_great_wall(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::OrigamiCrane,
        tr3("LANTERNA DE JADE!", "BALANCE OF MOUNT WUDANG!", "¡LINTERNA DE JADE!"),
        tr3("LIAN DEFENDE A GRANDE MURALHA", "LIAN GUARDS THE GREAT WALL", "LIAN PROTEGE LA GRAN MURALLA"),
        [0.24, 0.94, 0.62],
        side,
    )
}

pub(crate) fn bruno_daichi() -> ActiveEgg {
    ribbon(
        EggSprite::ShieldFortress,
        tr3("CHOQUE DE GIGANTES!", "IRON RING VS ALTAI WOLF!", "¡CHOQUE DE GIGANTES!"),
        tr3("FORÇA PURA NA ARENA!", "PURE HEAVYWEIGHT POWER!", "¡PURA FUERZA PESO PESADO!"),
        [1.0, 0.62, 0.24],
        0,
    )
}

pub(crate) fn garan_volcano(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::EclipseMoon,
        tr3("O PORTÃO DE CINZAS SE ABRE!", "THE ASHEN GATE OPENS!", "¡EL PORTÓN DE CENIZAS SE ABRE!"),
        tr3("GARAN DESPERTA NO VULCÃO DO SELO", "GARAN RETURNS TO HIS SEALED CRATER", "GARAN REGRESA AL CRÁTER DEL SELLO"),
        [0.78, 0.28, 1.0],
        side,
    )
}

pub(crate) fn valdric_oboro() -> ActiveEgg {
    ribbon(
        EggSprite::CrossedBlades,
        tr3("DUAS COROAS, UM TRONO!", "TWO CROWNS, ONE THRONE!", "¡DOS CORONAS, UN TRONO!"),
        tr3("IMPÉRIO RUBRO VS ECLIPSE ESCARLATE", "RED STAR EMPIRE VS SCARLET ECLIPSE", "IMPERIO ROJO VS ECLIPSE ESCARLATA"),
        [0.98, 0.24, 0.34],
        0,
    )
}

pub(crate) fn suzume_jin() -> ActiveEgg {
    ribbon(
        EggSprite::CrossedBlades,
        tr3("PISCOU, PERDEU!", "BLINK AND YOU'LL MISS ME!", "¡PARPADEAS Y PIERDES!"),
        tr3("VELOCIDADE DE IGA VS VALE DO VENTO", "IGA NINJA VS WIND VALLEY SCOUT", "NINJA DE IGA VS EXPLORADOR DEL VIENTO"),
        [0.94, 0.44, 0.98],
        0,
    )
}

pub(crate) fn volta_haruki(side: usize) -> ActiveEgg {
    speech(
        "VOLTA",
        tr3("Consulta: o que faz um coração ser corajoso?", "Query: what makes a heart courageous?", "Consulta: ¿qué hace valiente a un corazón?"),
        [0.44, 0.94, 1.0],
        side,
    )
}

pub(crate) fn akio_kaito(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::RamenBowl,
        tr3("QUEM PERDER PAGA O RAMEN!", "LOSER BUYS RAMEN!", "¡QUIEN PIERDA PAGA EL RAMEN!"),
        tr3("RIVALIDADE CLÁSSICA: AKIO VS KAITO", "CLASSIC RIVALS: AKIO VS KAITO", "RIVALES CLÁSICOS: AKIO VS KAITO"),
        [1.0, 0.72, 0.26],
        side,
    )
}

pub(crate) fn marlow_parrot(side: usize) -> ActiveEgg {
    flyby(
        EggSprite::PirateParrot,
        tr3("ARRR! TESOURO À VISTA!", "ARRR! TREASURE AHEAD!", "¡ARRR! ¡TESORO A LA VISTA!"),
        tr3("O PAPAGAIO DO EL VIENTO SALADO!", "EL VIENTO SALADO'S FIRST MATE!", "¡EL LORO DE EL VIENTO SALADO!"),
        [1.0, 0.36, 0.32],
        side == 1,
    )
}

pub(crate) fn thug_complaint(name: &str, side: usize) -> ActiveEgg {
    speech(
        name,
        tr3("O chefe não me paga o suficiente pra isso!", "The boss doesn't pay me enough for this!", "¡El jefe no me paga suficiente para esto!"),
        [1.0, 0.62, 0.28],
        side,
    )
}
