macro_rules! catalog {
    ($($key:literal => $en:literal, $pt:literal;)*) => {
        pub(super) const EN: &[(&str, &str)] = &[$(($key, $en)),*];
        pub(super) const PT: &[(&str, &str)] = &[$(($key, $pt)),*];
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
    "menu.mugen" => "Import M.U.G.E.N", "Importar M.U.G.E.N";
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
    "desc.mugen" => "Import M.U.G.E.N characters and stages from a .zip", "Importe personagens e cenários M.U.G.E.N de um .zip";
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
    "select.filter_bosses" => "BOSSES", "CHEFES";

    "results.rematch" => "Rematch", "Revanche";
    "results.replay" => "Replay", "Replay";
    "results.characters" => "Characters", "Personagens";
    "results.menu" => "Menu", "Menu";
    "results.arcade_complete" => "ARCADE COMPLETE!", "ARCADE COMPLETO!";
    "results.time" => "TIME", "TEMPO";
    "results.best_combo" => "BEST COMBO", "MAIOR COMBO";
    "results.damage" => "DAMAGE", "DANO";
    "results.life_left" => "LIFE LEFT", "VIDA RESTANTE";
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
