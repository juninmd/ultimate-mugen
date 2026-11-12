use super::*;
use crate::i18n::tr;
use crate::i18n::tr3;
use crate::update::State;

const PANEL: (f32, f32, f32, f32) = (760.0, 24.0, 496.0, 150.0);
const ACCEPT: (f32, f32, f32, f32) = (784.0, 118.0, 236.0, 40.0);
const LATER: (f32, f32, f32, f32) = (1032.0, 118.0, 200.0, 40.0);

impl App {
    fn update_line(&self) -> String {
        match self.updater.state().clone() {
            State::Checking => tr("Verificando atualizações…", "Checking for updates…").to_string(),
            State::Available(update) if update.binary.is_some() => format!(
                "{} v{}",
                tr3(
                    "Nova versão disponível:",
                    "New version available:",
                    "Nueva versión disponible:"
                ),
                update.version
            ),
            State::Available(update) => format!(
                "{} · v{} · {}",
                crate::update::build_label(),
                update.version,
                crate::update::RELEASES_URL
            ),
            State::Downloading {
                version,
                done,
                total,
            } => {
                let percent = if total > 0 {
                    done.saturating_mul(100) / total
                } else {
                    0
                };
                format!(
                    "{} v{} · {percent}%",
                    tr("Baixando atualização", "Downloading update"),
                    version
                )
            }
            State::Ready { version, .. } => format!(
                "{} v{} — {}",
                tr("Atualização", "Update"),
                version,
                tr("pronta — reiniciando…", "ready — restarting…")
            ),
            State::Failed(error) => {
                format!("{}: {error}", tr("Falha na atualização", "Update failed"))
            }
            State::Idle => crate::update::build_label(),
        }
    }

    pub(super) fn draw_update_status(&self) {
        let color = if matches!(self.updater.state(), State::Failed(_)) {
            hex("#ff8a9b")
        } else {
            hex("#a49cc0")
        };
        self.ui.text(
            &self.update_line(),
            VW - 14.0,
            VH - 12.0,
            14.0,
            color,
            Align::Right,
            false,
        );
    }

    /// O aviso só existe com uma versão encontrada, com binário para esta plataforma e sem o
    /// `--no-auto-update`; fora do título/menu ele espera (o jogo não pode parar no meio da luta).
    fn update_prompt_open(&self) -> bool {
        self.updater.prompt_allowed()
            && matches!(self.screen, Screen::Title | Screen::Menu { .. })
            && matches!(self.updater.state(), State::Available(update) if update.binary.is_some())
    }

    /// Clique dentro do painel não pode virar "start" no título.
    pub(super) fn update_prompt_takes_click(&self) -> bool {
        self.update_prompt_open() && self.hover_rect(PANEL.0, PANEL.1, PANEL.2, PANEL.3)
    }

    fn update_button(&self, rect: (f32, f32, f32, f32), label: &str, primary: bool) -> bool {
        let (x, y, w, h) = rect;
        let hot = self.hover_rect(x, y, w, h);
        let bg = match (primary, hot) {
            (true, true) => hex("#ff4d6d"),
            (true, false) => Color::new(0.85, 0.20, 0.32, 0.90),
            (false, true) => Color::new(1.0, 1.0, 1.0, 0.22),
            (false, false) => Color::new(1.0, 1.0, 1.0, 0.10),
        };
        self.ui.rect(x, y, w, h, bg);
        self.ui
            .text_bold(label, x + w / 2.0, y + 26.0, 17.0, WHITE, hex("#12051f"), Align::Center);
        self.clicked_rect(x, y, w, h)
    }

    fn draw_update_prompt(&mut self) {
        if !self.update_prompt_open() {
            return;
        }
        let State::Available(update) = self.updater.state() else {
            return;
        };
        let version = update.version.clone();
        let (x, y, w, h) = PANEL;
        crate::menu_art::glass(&self.ui, x, y, w, h, 1.0);
        self.ui.text_bold(
            tr3(
                "Atualização disponível",
                "Update available",
                "Actualización disponible",
            ),
            x + 24.0,
            y + 36.0,
            22.0,
            WHITE,
            hex("#12051f"),
            Align::Left,
        );
        let facts = format!(
            "{} v{} · {} v{} · {}",
            tr3("Instalada", "Installed", "Instalada"),
            crate::update::VERSION,
            tr3("nova", "new", "nueva"),
            version,
            tr3(
                "baixa só se você aceitar",
                "downloads only if you accept",
                "descarga solo si aceptas"
            )
        );
        self.ui.text(&facts, x + 24.0, y + 70.0, 16.0, hex("#a49cc0"), Align::Left, false);
        if self.update_button(
            ACCEPT,
            tr3("Atualizar agora", "Update now", "Actualizar ahora"),
            true,
        ) {
            self.updater.accept();
            self.audio.sfx("confirm");
        } else if self.update_button(
            LATER,
            tr3("Agora não", "Not now", "Ahora no"),
            false,
        ) {
            self.updater.dismiss();
            self.audio.sfx("select");
        }
    }

    /// Só instala o que o usuário aceitou: `Ready` nasce exclusivamente de `Updater::accept`.
    /// Chega sem aceite quando o pacote já veio de um boot anterior — aí vira aviso de novo.
    pub(super) fn apply_update_if_ready(&mut self) {
        if !matches!(self.screen, Screen::Title | Screen::Menu { .. }) {
            return;
        }
        if let Some((version, staged)) = self.updater.state().ready() {
            let (version, staged) = (version.to_string(), staged.to_path_buf());
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            match crate::update::install(&staged) {
                Ok(exe) => match crate::update::relaunch(&exe) {
                    Ok(()) => {
                        self.msg = tr(
                            "Atualização aplicada — reiniciando…",
                            "Update applied — restarting…",
                        )
                        .to_string();
                        self.updater.set_state(State::Idle);
                        self.quit = true;
                    }
                    Err(error) => {
                        self.msg = error;
                        self.updater.set_state(State::Idle);
                    }
                },
                Err(error) => {
                    self.msg = format!("{}: {error}", tr("Falha na atualização", "Update failed"));
                    self.updater.set_state(State::Failed(error));
                }
            }
            let _ = (version, staged);
            return;
        }
        self.draw_update_prompt();
    }
}
