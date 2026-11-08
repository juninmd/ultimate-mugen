//! UI localization: a keyed string table with `en` (default) and `pt-BR` catalogs.
//! Lookup order is current locale -> English -> the key itself; a missing key is logged once.
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Pt,
    En,
}

pub const LANGS: [(Lang, &str); 2] = [(Lang::En, "English"), (Lang::Pt, "Português (Brasil)")];

static CURRENT: AtomicU8 = AtomicU8::new(1);
static MISSING: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

pub fn set(l: Lang) {
    CURRENT.store(if l == Lang::En { 1 } else { 0 }, Ordering::Relaxed);
}
pub fn get() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == 1 { Lang::En } else { Lang::Pt }
}
pub fn default_lang() -> Lang { Lang::En }

/// Legacy helper: pick one of two literals. Prefer `t(key)` in new code.
pub fn tr<'a>(pt: &'a str, en: &'a str) -> &'a str {
    if get() == Lang::En { en } else { pt }
}

/// Translate a catalog key. Unknown keys fall back to English, then to the key itself.
pub fn t(key: &'static str) -> &'static str {
    let table = if get() == Lang::En { EN } else { PT };
    if let Some((_, v)) = table.iter().find(|(k, _)| *k == key) {
        return v;
    }
    if let Some((_, v)) = EN.iter().find(|(k, _)| *k == key) {
        return v;
    }
    let mut missing = MISSING.lock().unwrap_or_else(|e| e.into_inner());
    if missing.insert(key) {
        eprintln!("i18n: missing key '{key}'");
    }
    key
}

/// Translate a key that contains `{name}` placeholders and fill them in.
pub fn tf(key: &'static str, pairs: &[(&str, &str)]) -> String {
    let mut out = t(key).to_string();
    for (k, v) in pairs {
        out = out.replace(k, v);
    }
    out
}

/// Keys without an English entry (used by tests and the release checklist).
pub fn keys_without_english() -> Vec<&'static str> {
    PT.iter()
        .map(|(k, _)| *k)
        .filter(|k| !EN.iter().any(|(e, _)| e == k))
        .collect()
}

pub fn missing_logged() -> Vec<&'static str> {
    MISSING
        .lock()
        .map(|m| m.iter().copied().collect())
        .unwrap_or_default()
}

macro_rules! catalog {
    ($($key:literal => $en:literal, $pt:literal;)*) => {
        const EN: &[(&str, &str)] = &[$(($key, $en)),*];
        const PT: &[(&str, &str)] = &[$(($key, $pt)),*];
    };
}

catalog! {
    "menu.arcade" => "Arcade", "Arcade";
    "menu.survival" => "Survival", "Sobrevivência";
    "menu.cpu" => "Versus CPU", "Versus CPU";
    "menu.versus" => "2 Players", "2 Jogadores";
    "menu.training" => "Training", "Treino";
    "menu.watch" => "CPU vs CPU", "CPU vs CPU";
    "menu.team_seq" => "Team · Sequence", "Equipe · Sequência";
    "menu.team_simul" => "Team · Parallel", "Equipe · Paralelo";
    "menu.team_tag" => "Team · Tag", "Equipe · Tag";
    "menu.mugen" => "M.U.G.E.N characters", "Personagens MUGEN";
    "menu.market" => "GitHub marketplace", "Mercado GitHub";
    "menu.controls" => "Controls", "Controles";
    "menu.settings" => "Settings", "Configurações";
    "menu.replays" => "Replays", "Replays";
    "menu.quit" => "Quit", "Sair";

    "desc.arcade" => "Fight a sequence of opponents", "Enfrente uma sequência de oponentes";
    "desc.survival" => "Endless opponents, life refilled after each win, score", "Oponentes sem fim, vida recuperada a cada vitória e pontuação";
    "desc.cpu" => "You against the computer", "Você contra o computador";
    "desc.versus" => "Shared keyboard or two controllers", "Teclado compartilhado ou dois controles";
    "desc.training" => "Training dummy, infinite bar, hitboxes", "Boneco de treino, barra infinita, hitboxes";
    "desc.watch" => "Watch the AI fight", "Assista a IA lutar";
    "desc.team_seq" => "Up to 5v5, one at a time; the winner keeps its life", "Até 5 contra 5, um de cada vez; o vencedor mantém a vida";
    "desc.team_simul" => "5v5 at once (2 to 5) · Tab switches fighter", "5 contra 5 ao mesmo tempo (2 a 5) · TAB troca de lutador";
    "desc.team_tag" => "One on stage, the partner waits and tags in (2 to 3 per side)", "Um em campo, o parceiro espera e entra na troca (2 a 3 por lado)";
    "desc.mugen" => "How to add characters and stages", "Como adicionar personagens e cenários";
    "desc.market" => "Download characters, stages and music from repositories", "Baixe personagens, cenários e músicas de repositórios";
    "desc.controls" => "Keyboard, Xbox, PlayStation, touch and rumble", "Teclado, Xbox, PlayStation, toque e vibração";
    "desc.settings" => "Video, audio, controls, gameplay and language", "Vídeo, áudio, controles, jogabilidade e idioma";
    "desc.replays" => "Watch, rename and delete recorded fights", "Assista, renomeie e apague lutas gravadas";
    "desc.quit" => "Close the game", "Fechar o jogo";

    "mode.arcade" => "ARCADE", "ARCADE";
    "mode.survival" => "SURVIVAL", "SOBREVIVÊNCIA";
    "mode.cpu" => "VERSUS CPU", "VERSUS CPU";
    "mode.versus" => "2 PLAYERS", "2 JOGADORES";
    "mode.training" => "TRAINING", "TREINO";
    "mode.watch" => "CPU vs CPU", "CPU vs CPU";
    "mode.team_seq" => "TEAM · SEQUENCE", "EQUIPE · SEQUÊNCIA";
    "mode.team_simul" => "TEAM · PARALLEL", "EQUIPE · PARALELO";
    "mode.team_tag" => "TEAM · TAG", "EQUIPE · TAG";

    "select.p1" => "PLAYER 1: choose your fighter", "JOGADOR 1: escolha seu lutador";
    "select.p2" => "PLAYER 2: choose", "JOGADOR 2: escolha";
    "select.partner" => "PLAYER 2 (PARTNER): choose", "JOGADOR 2 (PARCEIRO): escolha";
    "select.opponent" => "Choose the opponent", "Escolha o oponente";
    "select.team_hint" => "Build your team · Q/E: size", "Monte sua equipe · Q/E: tamanho";
    "select.coop_on" => "CO-OP 2P: ON [V]", "CO-OP 2P: LIGADO [V]";
    "select.coop_off" => "CO-OP 2P: OFF [V]", "CO-OP 2P: DESLIGADO [V]";
    "select.palette" => "PALETTE", "PALETA";
    "select.palette_hint" => ", . cycle · P random", ", . ciclo · P aleatória";
    "select.stage" => "STAGE", "CENÁRIO";
    "select.random" => "Random", "Aleatório";
    "select.choose_stage" => "CHOOSE THE STAGE", "ESCOLHA O CENÁRIO";
    "select.filter_all" => "ALL", "TODOS";
    "select.filter_originals" => "ORIGINALS", "ORIGINAIS";
    "select.filter_mugen" => "M.U.G.E.N", "M.U.G.E.N";

    "results.rematch" => "Rematch", "Revanche";
    "results.replay" => "Replay", "Replay";
    "results.characters" => "Characters", "Personagens";
    "results.menu" => "Menu", "Menu";
    "results.arcade_complete" => "ARCADE COMPLETE!", "ARCADE COMPLETO!";
    "results.wins" => "{name} WINS!", "{name} VENCE!";
    "results.team_wins" => "TEAM {name} WINS!", "TIME {name} VENCEU!";
    "boss.unlocked_oboro" => "★ NEW FIGHTER UNLOCKED: OBORO! ★", "★ NOVO PERSONAGEM DESBLOQUEADO: OBORO! ★";
    "boss.cleared" => "★ ARCADE CLEARED! ★", "★ MODO HISTÓRIA ZERADO! ★";
    "boss.next_villain" => "★ NEW MAIN VILLAIN: {boss} ★", "★ NOVO VILÃO PRINCIPAL: {boss} ★";
    "pause.title" => "PAUSED", "PAUSA";
    "pause.continue" => "Continue", "Continuar";
    "pause.hitboxes" => "Hitboxes", "Hitboxes";
    "pause.restart" => "Restart fight", "Reiniciar luta";
    "pause.change" => "Change fighters", "Trocar personagens";
    "pause.settings" => "Settings", "Configurações";
    "pause.menu" => "Main menu", "Menu principal";
    "pause.moves" => "MOVES", "GOLPES";
    "pause.no_moves" => "This character does not define commands in the .cmd", "Este personagem não define comandos no .cmd";
    "pause.page" => "page", "página";

    "replays.title" => "REPLAYS", "REPLAYS";
    "replays.empty" => "No recorded fight yet.", "Nenhuma luta gravada ainda.";
    "replays.empty_hint" => "Finish a fight and it will show up here.", "Termine uma luta para ela aparecer aqui.";
    "replays.play" => "Play", "Jogar";
    "replays.rename" => "Rename", "Renomear";
    "replays.delete" => "Delete", "Apagar";
    "replays.sort" => "Sort by name", "Ordenar por nome";
    "replays.back" => "Back", "Voltar";
    "replays.rename_title" => "RENAME REPLAY", "RENOMEAR REPLAY";
    "replays.confirm" => "Press Enter to confirm · Esc cancels", "Enter confirma · Esc cancela";
    "replays.confirm_delete" => "Press DELETE again to delete", "Pressione DELETE de novo para apagar";
    "replays.renamed" => "Replay renamed", "Replay renomeado";
    "replays.deleted" => "Replay deleted", "Replay apagado";
    "replays.paused" => "PAUSED", "PAUSADO";
    "replays.bar" => "REPLAY {speed}x · {tick}/{total} frames · SPACE {action} · 1/2/4 speed · . step · ESC exits", "REPLAY {speed}x · {tick}/{total} quadros · ESPAÇO {action} · 1/2/4 velocidade · . avança · ESC sai";
    "replays.resume" => "resume", "continua";
    "replays.pause" => "pause", "pausa";

    "survival.over" => "GAME OVER", "FIM DE JOGO";
    "survival.points" => "{score} POINTS", "{score} PONTOS";
    "survival.wins" => "{count} fight(s) won", "{count} luta(s) vencida(s)";
    "survival.record" => "BEST: {best}", "RECORDE: {best}";
    "survival.new_record" => "NEW RECORD!", "NOVO RECORDE!";
    "survival.retry" => "Try again", "Tentar de novo";
    "survival.watch" => "Watch replay", "Ver replay";
    "survival.hud" => "SURVIVAL · ROUND {round} · {score} PTS", "SOBREVIVÊNCIA · RODADA {round} · {score} PTS";

    "controls.title" => "CONTROLS", "CONTROLES";
    "controls.back" => "Back", "Voltar";
    "controls.customize" => "Customize keyboard (Tab)", "Personalizar teclado (Tab)";
    "controls.move" => "Move", "Mover";
    "controls.punch" => "Punch", "Soco";
    "controls.kick" => "Kick", "Chute";
    "controls.special" => "Special", "Especial";
    "controls.super" => "Super", "Super";
    "controls.awaken" => "Awaken", "Despertar";
    "controls.pause" => "Pause", "Pausa";
    "controls.keyboard" => "Keyboard", "Teclado";
    "controls.touch" => "Touch", "Toque";

    "controls.none" => "No controller connected", "Nenhum controle conectado";
    "common.back" => "Back", "Voltar";
    "common.ok" => "OK", "OK";
    "common.cancel" => "Cancel", "Cancelar";
    "common.loading" => "Loading", "Carregando";
}

#[cfg(test)]
mod tests {
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

    /// Grep guard: the migrated screens must not carry Portuguese literals anymore.
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
}
