use super::*;

impl App {
    pub(super) fn gameplay_rows(&self) -> Vec<Row> {
        let s = &self.settings;
        vec![
            row(
                Act::Difficulty,
                tr("Dificuldade da CPU", "CPU difficulty"),
                format!("{} / 8", s.difficulty),
                tr(
                    "Vale para Arcade, Versus CPU e Equipes",
                    "Applies to Arcade, Versus CPU and Teams",
                ),
            ),
            row(
                Act::RoundTime,
                tr("Tempo do round", "Round time"),
                match ROUND_SECONDS[s.round_time.min(ROUND_SECONDS.len() - 1)] {
                    0 => tr("Infinito", "Infinite").to_string(),
                    n => format!("{n} s"),
                },
                tr(
                    "Vale a partir da próxima luta",
                    "Takes effect from the next fight",
                ),
            ),
            row(
                Act::CharTuning,
                tr("Ajustes de personagens", "Character tuning"),
                tr("Abrir", "Open"),
                tr("Vida, dano, defesa, velocidades, gravidade e barra de poder", "Life, damage, defence, speeds, gravity and power bar"),
            ),
            row(
                Act::Rounds,
                tr("Rounds para vencer", "Rounds to win"),
                format!(
                    "{} {}",
                    tr("Melhor de", "Best of"),
                    ROUNDS_TO_WIN[s.rounds.min(ROUNDS_TO_WIN.len() - 1)] * 2 - 1
                ),
                tr(
                    "Vitórias necessárias: 1, 2 ou 3 rounds",
                    "Wins needed: 1, 2 or 3 rounds",
                ),
            ),
            row(
                Act::Life,
                tr("Vida dos lutadores", "Fighter life"),
                format!("{}%", LIFE_PERCENT[s.life.min(LIFE_PERCENT.len() - 1)]),
                tr(
                    "Multiplica a barra de vida de todos",
                    "Scales everyone's life bar",
                ),
            ),
            row(
                Act::SingleRound,
                tr("Round único", "Single round"),
                on_off(s.single_round),
                tr(
                    "A luta termina no primeiro round vencido",
                    "The fight ends on the first round won",
                ),
            ),
            row(
                Act::ClassicPushbox,
                tr("Pushbox clássico", "Classic pushbox"),
                on_off(s.classic_pushbox),
                tr(
                    "O atacante não é empurrado pelo defensor (regra M.U.G.E.N)",
                    "The attacker is not pushed back by the defender (M.U.G.E.N rule)",
                ),
            ),
            row(
                Act::Knockdown,
                tr3("Quedas ao apanhar", "Knockdowns on hit", "Caídas al recibir golpes"),
                knockdown_label(s.knockdown),
                tr3(
                    "Golpes fracos só empurram; só golpes fortes, supers e combos longos derrubam",
                    "Light hits only push back; only strong hits, supers and long combos knock down",
                    "Los golpes débiles solo empujan; solo los fuertes, supers y combos largos derriban",
                ),
            ),
            row(
                Act::TeamLifeShare,
                tr("Vida compartilhada em equipe", "Shared team life"),
                on_off(s.team_life_share),
                tr(
                    "O vencedor mantém a vida para o próximo membro",
                    "The winner keeps life for the next member",
                ),
            ),
            row(
                Act::GameSpeed,
                tr("Velocidade do jogo", "Game speed"),
                format!(
                    "{:.0}%",
                    GAME_SPEEDS[s.game_speed.min(GAME_SPEEDS.len() - 1)] * 100.0
                ),
                tr(
                    "Ritmo da simulação, sem alterar o conteúdo",
                    "Simulation pace, without changing content",
                ),
            ),
            row(
                Act::SurvivalRecovery,
                tr(
                    "Vida recuperada (Sobrevivência)",
                    "Life recovered (Survival)",
                ),
                format!(
                    "{}%",
                    SURVIVAL_RECOVERY[s.survival_recovery.min(SURVIVAL_RECOVERY.len() - 1)]
                ),
                tr(
                    "Quanto da barra volta depois de cada vitória",
                    "How much of the bar comes back after each win",
                ),
            ),
            row(Act::ShowBoxes, tr3("Exibir Hitboxes / Hurtboxes", "Show Hitboxes / Hurtboxes", "Mostrar Hitboxes / Hurtboxes"), on_off(s.show_boxes), tr3("Mostra caixas de ataque e colisão durante as lutas", "Displays attack and body collision boxes during fights", "Muestra cajas de ataque y colisión durante los combates")),
            row(Act::SkipIntros, tr3("Pular intros dos lutadores", "Skip fighter intros", "Saltar intros de luchadores"), on_off(s.skip_intros), tr3("Começa o round direto, sem a pose de apresentação (qualquer botão ou toque também pula)", "Starts the round straight away, no face-off pose (any button or tap also skips)", "Empieza el round directo, sin la pose de presentación (cualquier botón o toque también salta)")),
            row(Act::StoryEnabled, tr3("Modo História (TOML)", "Story mode (TOML)", "Modo Historia (TOML)"), on_off(s.story_enabled && crate::story::active_script().campaign.enabled), tr3("Ative/desative ou personalize em story/campaign.toml", "Enable/disable or customize via story/campaign.toml", "Activa/desactiva o personaliza en story/campaign.toml")),
                row(Act::StoryStudio, tr3("Editor da História", "Story Studio", "Editor de Historia"), tr3("Abrir", "Open", "Abrir"), tr3("Edite regras e falas da campanha (salvas em TOML) dentro do jogo", "Edit campaign rules and dialogue lines (saved as TOML) in-game", "Edita reglas y diálogos de la campaña (guardados en TOML) dentro del juego")),
        ]
    }
}
