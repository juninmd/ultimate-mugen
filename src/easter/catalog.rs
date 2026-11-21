use super::builders::{corner_pop, floor_roll, flyby, ribbon};
use super::{ActiveEgg, EggSprite};
use crate::i18n::tr3;

pub(crate) fn toasty() -> ActiveEgg {
    corner_pop(
        EggSprite::ToastyRef,
        tr3("QUENTINHO!", "TOASTY!", "¡TOSTADITO!"),
        tr3("IMPACTO LENDÁRIO!", "LEGENDARY LAUNCH!", "¡IMPACTO LEGENDARIO!"),
        [1.0, 0.36, 0.24],
    )
}

pub(crate) fn double_ko() -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("EMPATE ÉPICO!", "DOUBLE K.O. DRAMA!", "¡EMPATE ÉPICO!"),
        tr3("DOIS GUERREIROS CAEM JUNTOS", "BOTH WARRIORS FALL TOGETHER", "AMBOS CAEN AL MISMO TIEMPO"),
        [1.0, 0.78, 0.2],
        0,
    )
}

pub(crate) fn magic_pixel(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("PIXEL MÁGICO!", "MAGIC PIXEL!", "¡PÍXEL MÁGICO!"),
        tr3("SOBREVIVEU POR UM FIO!", "HANGING BY ONE PIXEL!", "¡SOBREVIVIÓ POR UN HILO!"),
        [1.0, 0.28, 0.56],
        side,
    )
}

pub(crate) fn flawless_perfect(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("INTOCÁVEL!", "UNTOUCHABLE!", "¡INTOCABLE!"),
        tr3("VITÓRIA PERFEITA 100%!", "FLAWLESS 100% ROUND!", "¡RONDA PERFECTA AL 100%!"),
        [1.0, 0.86, 0.24],
        side,
    )
}

pub(crate) fn mutual_clash() -> ActiveEgg {
    ribbon(
        EggSprite::ShieldFortress,
        tr3("CHOQUE DE TITÃS!", "CLASH OF FATES!", "¡CHOQUE DE TITANES!"),
        tr3("TROCA SIMULTÂNEA!", "SIMULTANEOUS TRADE!", "¡INTERCAMBIO SIMULTÁNEO!"),
        [0.32, 0.86, 1.0],
        0,
    )
}

pub(crate) fn taunt_bodied() -> ActiveEgg {
    corner_pop(
        EggSprite::ToastyRef,
        tr3("PROVOCOU, LEVOU!", "TAUNT TO GET BODIED!", "¡PROVOCÓ Y COBRÓ!"),
        tr3("NUNCA SUBESTIME O RIVAL", "CLASSIC FIGHTING GAME RULE", "REGLA CLÁSICA DE ARCADE"),
        [1.0, 0.52, 0.2],
    )
}

pub(crate) fn honorable_taunt() -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("RESPEITO MÚTUO!", "HONORABLE DUEL!", "¡RESPETO MUTUO!"),
        tr3("AMBOS PROVOCARAM COM HONRA", "BOTH FIGHTERS SALUTED", "AMBOS SALUDARON CON HONOR"),
        [0.98, 0.68, 0.84],
        0,
    )
}

pub(crate) fn sync_jump() -> ActiveEgg {
    ribbon(
        EggSprite::ArcadeCoin,
        tr3("NO MESMO RITMO!", "SYNC JUMP!", "¡SALTO SINCRONIZADO!"),
        tr3("MENTES CONECTADAS!", "GREAT MINDS JUMP ALIKE!", "¡MENTES CONECTADAS!"),
        [0.44, 0.92, 0.72],
        0,
    )
}

pub(crate) fn crouch_party(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::ArcadeCoin,
        tr3("AGACHAMENTO TÁTICO!", "TACTICAL DANCE PARTY!", "¡BAILE TÁCTICO!"),
        tr3("MINDGAMES DE ARCADE!", "PURE ARCADE MINDGAMES!", "¡PURO MINDGAME ARCADE!"),
        [0.86, 0.44, 1.0],
        side,
    )
}

pub(crate) fn speedrun_ko(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::Stopwatch,
        tr3("SPEEDRUN K.O.!", "SPEEDRUN K.O.!", "¡K.O. RELÁMPAGO!"),
        tr3("EM MENOS DE 10 SEGUNDOS!", "UNDER 10 SECONDS!", "¡EN MENOS DE 10 SEGUNDOS!"),
        [0.24, 0.92, 1.0],
        side,
    )
}

pub(crate) fn buzzer_beater() -> ActiveEgg {
    ribbon(
        EggSprite::Stopwatch,
        tr3("NO ESTOURAR DO RELÓGIO!", "BUZZER BEATER!", "¡EN EL ÚLTIMO SEGUNDO!"),
        tr3("DECISÃO NOS SEGUNDOS FINAIS!", "CLUTCH FINAL SECONDS!", "¡DECISIÓN AL LÍMITE!"),
        [1.0, 0.4, 0.3],
        0,
    )
}

pub(crate) fn air_juggle(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::ArcadeCoin,
        tr3("SEM GRAVIDADE!", "GRAVITY OFF!", "¡SIN GRAVEDAD!"),
        tr3("MALABARISMO AÉREO!", "AIR JUGGLE MASTER!", "¡MAESTRO DEL JUGGLE AÉREO!"),
        [0.36, 0.84, 1.0],
        side,
    )
}

pub(crate) fn guard_fortress(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::ShieldFortress,
        tr3("MURALHA DE FERRO!", "IRON FORTRESS!", "¡MURALLA DE HIERRO!"),
        tr3("BLOQUEIO IMPENETRÁVEL!", "6+ HITS BLOCKED CLEAN!", "¡6+ GOLPES BLOQUEADOS!"),
        [0.42, 0.72, 1.0],
        side,
    )
}

pub(crate) fn guard_shatter(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::ShieldFortress,
        tr3("DEFESA EM PEDAÇOS!", "GUARD SHATTERED!", "¡DEFENSA DESTROZADA!"),
        tr3("QUEBRA TOTAL!", "ARMOR CRACKED WIDE OPEN!", "¡BRECHA TOTAL!"),
        [1.0, 0.32, 0.32],
        side,
    )
}

pub(crate) fn double_super_clash() -> ActiveEgg {
    ribbon(
        EggSprite::ToastyRef,
        tr3("COLISÃO DE SUPERS!", "DOUBLE SUPER CLASH!", "¡COLISIÓN DE SUPERS!"),
        tr3("ENERGIA NO LIMITE MÁXIMO!", "MAXIMUM OVERDRIVE!", "¡ENERGÍA AL MÁXIMO!"),
        [1.0, 0.72, 0.16],
        0,
    )
}

pub(crate) fn awakening_duel() -> ActiveEgg {
    ribbon(
        EggSprite::CrownPerfect,
        tr3("DESPERTAR DUPLO!", "LIMIT BREAK DUEL!", "¡DESPERTAR DOBLE!"),
        tr3("PODER TOTAL DOS DOIS LADOS!", "BOTH FIGHTERS AWAKENED!", "¡PODER TOTAL EN AMBOS LADOS!"),
        [0.92, 0.36, 1.0],
        0,
    )
}

pub(crate) fn overkill_combo() -> ActiveEgg {
    corner_pop(
        EggSprite::ToastyRef,
        tr3("CHEGA, JÁ DEU!", "STOP, HE'S ALREADY DOWN!", "¡BASTA, YA FUE!"),
        tr3("COMBO MONSTRUOSO 15+!", "15+ HIT OVERKILL COMBO!", "¡COMBO MONSTRUOSO 15+!"),
        [1.0, 0.24, 0.36],
    )
}

pub(crate) fn corner_escape(side: usize) -> ActiveEgg {
    ribbon(
        EggSprite::ShieldFortress,
        tr3("FUGA DE MESTRE!", "GREAT CORNER ESCAPE!", "¡GRAN ESCAPE!"),
        tr3("VIROU O JOGO NA PAREDE!", "REVERSED FROM THE WALL!", "¡REVIVIÓ DESDE LA ESQUINA!"),
        [0.34, 0.96, 0.62],
        side,
    )
}

pub(crate) fn staredown() -> ActiveEgg {
    floor_roll(
        EggSprite::Tumbleweed,
        tr3("DUELO DE OLHARES...", "HIGH NOON STAREDOWN...", "DUELO DE MIRADAS..."),
        tr3("QUEM PISCAR PRIMEIRO PERDE", "WHO BLINKS FIRST?", "¿QUIÉN PARPADEA PRIMERO?"),
        [0.92, 0.72, 0.42],
        false,
    )
}

pub(crate) fn max_meter_coin(side: usize) -> ActiveEgg {
    flyby(
        EggSprite::ArcadeCoin,
        tr3("FICHA MÁXIMA — 1UP!", "INSERT COIN — MAX METER!", "¡FICHA MÁXIMA — 1UP!"),
        tr3("SUPER BARRA NO TALO!", "SUPER GAUGE OVERFLOWING!", "¡BARRA SÚPER AL MÁXIMO!"),
        [1.0, 0.84, 0.22],
        side == 1,
    )
}
