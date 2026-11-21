use super::{stages_catalog as sc, FightTracker};
use crate::audio::Audio;
use crate::i18n::tr3;
use ume_engine::matchs::Match;

pub(crate) fn check(m: &Match, stage_name: &str, tr: &mut FightTracker, audio: &mut Audio) {
    if m.round_state != 2 {
        return;
    }
    let name = stage_name.to_lowercase();
    let t = m.round_ticks;
    let heavy_hit = took_heavy(m, tr, 95.0);
    let super_fired = spent_super(m, tr);
    let max_combo = m.roots().map(|f| f.combo_count).max().unwrap_or(0);
    let any_full_meter = m.roots().any(|f| f.power >= f.power_max.max(1.0));

    if name.contains("dojo") {
        if t == 260 {
            tr.trigger(20, false, sc::dojo_crane(), audio, "tick");
        }
        if heavy_hit {
            tr.trigger(21, true, sc::dojo_roof_rumble(), audio, "land");
        }
    } else if name.contains("neo-metr") || name.contains("neon") {
        if t == 240 {
            tr.trigger(22, false, sc::neon_ufo(), audio, "unlock");
        }
        if super_fired {
            tr.trigger(23, true, sc::neon_overload(), audio, "meter");
        }
    } else if name.contains("bambu") {
        if max_combo >= 4 {
            tr.trigger(24, true, sc::bamboo_spirits(), audio, "tick");
        }
    } else if name.contains("planalto") || name.contains("plains") {
        if t == 300 {
            tr.trigger(25, false, sc::plains_galleon(), audio, "tick");
        }
    } else if name.contains("espírit") || name.contains("espirit") || name.contains("spirit") {
        if super_fired || m.roots().any(|f| f.form > 0) {
            tr.trigger(26, false, sc::spirit_kitsune(), audio, "transform");
        }
    } else if name.contains("vulc") || name.contains("volcano") {
        if heavy_hit {
            tr.trigger(27, true, sc::volcano_geyser(), audio, "burst");
        }
    } else if name.contains("brasil") || name.contains("brazil") {
        if t == 220 {
            tr.trigger(28, false, sc::brazil_capybara(), audio, "confirm");
        }
        if m.round_no >= 2 && t == 140 {
            tr.trigger(29, false, sc::brazil_caramelo(), audio, "unlock");
        }
    } else if name.contains("cristo") || name.contains("corcovado") || name.contains("christ") {
        if t == 240 {
            tr.trigger(30, false, sc::christ_toucan(), audio, "confirm");
        }
    } else if name.contains("muralha") || name.contains("great wall") {
        if t == 260 {
            tr.trigger(31, false, sc::great_wall_kite(), audio, "tick");
        }
    } else if name.contains("petra") {
        if heavy_hit {
            tr.trigger(32, true, sc::petra_scarab(), audio, "meter");
        }
    } else if name.contains("texas") {
        if t == 240 {
            tr.trigger(33, false, sc::texas_noon(), audio, "tick");
        }
    } else if name.contains("chich") || name.contains("kukulc") {
        if super_fired {
            tr.trigger(34, true, sc::chichen_serpent(), audio, "super");
        }
    } else if name.contains("coliseu") || name.contains("colosseum") {
        if max_combo >= 6 {
            tr.trigger(35, true, sc::colosseum_laurel(), audio, "crowd");
        }
    } else if name.contains("taj") {
        if m.round_no >= 2 && t == 120 {
            tr.trigger(36, false, sc::taj_doves(), audio, "unlock");
        }
    } else if name.contains("giz") || name.contains("pyramids") {
        if any_full_meter {
            tr.trigger(37, true, sc::giza_horus(), audio, "meter");
        }
    } else if name.contains("babil") || name.contains("hanging") {
        if max_combo >= 4 && m.roots().any(|f| f.state_type == 'A') {
            tr.trigger(38, true, sc::babylon_peacock(), audio, "confirm");
        }
    } else {
        check_classical_wonders(&name, t, heavy_hit, super_fired, tr, audio);
    }
}

fn check_classical_wonders(name: &str, t: i32, heavy: bool, super_fired: bool, tr: &mut FightTracker, audio: &mut Audio) {
    if name.contains("zeus") && (heavy || super_fired) {
        tr.trigger(
            39,
            true,
            sc::ancient_wonder(
                tr3("RAIO DE ZEUS!", "ZEUS'S THUNDERBOLT!", "¡RAYO DE ZEUS!"),
                tr3("OLÍMPIA RESPONDE AO COMBATE", "OLYMPIA ANSWERS THE CLASH", "OLIMPIA RESPONDE AL CHOQUE"),
                [1.0, 0.86, 0.3],
            ),
            audio,
            "super",
        );
    } else if (name.contains("farol") || name.contains("alexandria")) && t == 250 {
        tr.trigger(
            39,
            false,
            sc::ancient_wonder(
                tr3("FEIXE DE PHAROS!", "PHAROS BEACON SWEEP!", "¡HAZ DE PHAROS!"),
                tr3("A LUZ DE ALEXANDRIA ILUMINA A ARENA", "ALEXANDRIA'S BEACON LIGHTS THE FIGHT", "EL FARO ILUMINA LA ARENA"),
                [0.98, 0.92, 0.62],
            ),
            audio,
            "unlock",
        );
    } else if (name.contains("ártemis") || name.contains("artemis")) && t == 250 {
        tr.trigger(
            39,
            false,
            sc::ancient_wonder(
                tr3("LUA DE ÁRTEMIS!", "SILVER CRESCENT OF ARTEMIS!", "¡LUNA DE ARTEMISA!"),
                tr3("CAÇADORA DE ÉFESO OBSERVA", "EPHESUS WATCHES FROM ABOVE", "ÉFESO OBSERVA DESDE LO ALTO"),
                [0.64, 0.92, 1.0],
            ),
            audio,
            "tick",
        );
    } else if (name.contains("rodes") || name.contains("rhodes")) && heavy {
        tr.trigger(
            39,
            true,
            sc::ancient_wonder(
                tr3("BRONZE DE RODES!", "COLOSSUS HARBOR ROAR!", "¡BRONCE DE RODAS!"),
                tr3("O GIGANTE DO PORTO ECOA!", "THE HARBOR TITAN ECHOES!", "¡EL COLOSO DEL PUERTO RESUENA!"),
                [0.92, 0.66, 0.32],
            ),
            audio,
            "crowd",
        );
    } else if (name.contains("halicarnasso") || name.contains("mausol")) && t == 250 {
        tr.trigger(
            39,
            false,
            sc::ancient_wonder(
                tr3("CHAMA DE HALICARNASSO!", "ETERNAL FLAME OF HALICARNASSUS!", "¡LLAMA DE HALICARNASO!"),
                tr3("MARAVILHA ANTIGA DESPERTA", "ANCIENT WONDER AWAKENS", "MARAVILLA ANTIGUA DESPIERTA"),
                [1.0, 0.78, 0.44],
            ),
            audio,
            "unlock",
        );
    }
}

fn took_heavy(m: &Match, tr: &FightTracker, threshold: f32) -> bool {
    (0..2).any(|side| {
        m.roots()
            .find(|f| f.side == side)
            .is_some_and(|f| tr.prev_life[side] - f.life >= threshold)
    })
}

fn spent_super(m: &Match, tr: &FightTracker) -> bool {
    (0..2).any(|side| {
        m.roots()
            .find(|f| f.side == side)
            .is_some_and(|f| tr.prev_power[side] - f.power >= 950.0)
    })
}
