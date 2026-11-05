use super::*;
use crate::i18n::tr;
use crate::update::State;

impl App {
    fn update_line(&self) -> String {
        match self.updater.state().clone() {
            State::Checking => tr("Verificando atualizações…", "Checking for updates…").to_string(),
            State::Available(update) if update.binary.is_some() => format!(
                "{} v{} · {}",
                tr("Nova versão disponível:", "New version available:"),
                update.version,
                tr("baixando…", "downloading…")
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

    pub(super) fn apply_update_if_ready(&mut self) {
        if !matches!(self.screen, Screen::Title | Screen::Menu { .. }) {
            return;
        }
        let Some((version, staged)) = self.updater.state().ready() else {
            return;
        };
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
    }
}
