use super::*;

impl App {
    /// Snapshot the running session as the replayable "last fight" (used by the screenshot tour's determinism check).
    pub fn capture_replay(&mut self) {
        if let Some(s) = self.session.as_ref() {
            self.last_fight = Some(Replay {
                sel: self.sel,
                mode: self.mode,
                team: self.team.clone(),
                rec: s.rec.clone(),
                rules: self.fight_rules,
                seed: s.seed,
                life: self.run_life(),
                pal: self.pal,
                team_pal: self.team_pal.clone(),
                net: s.netplay_config(),
            });
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub async fn start_netplay(&mut self, a: &crate::netlink::NetArgs) {
        let find_char = |id: &str| {
            self.reg
                .chars
                .iter()
                .position(|c| c.id.eq_ignore_ascii_case(id) || c.name.eq_ignore_ascii_case(id))
        };
        let (c1, c2) = match &a.chars {
            Some((x, y)) => (find_char(x), find_char(y)),
            None => (Some(0), Some(1.min(self.reg.chars.len() - 1))),
        };
        let (Some(c1), Some(c2)) = (c1, c2) else {
            self.msg = format!("Netplay: character not found ({:?})", a.chars);
            return;
        };
        let stage = a.stage.as_ref().map_or(Some(0), |id| {
            self.reg
                .stages
                .iter()
                .position(|s| s.id.eq_ignore_ascii_case(id) || s.name.eq_ignore_ascii_case(id))
        });
        let Some(stage) = stage else {
            self.msg = format!("Netplay: stage not found ({:?})", a.stage);
            return;
        };
        let link = match crate::netlink::NetLink::connect(a) {
            Ok(l) => l,
            Err(e) => {
                self.msg = format!("Netplay: {e}");
                return;
            }
        };
        self.mode = if a.spectate {
            Mode::Watch
        } else {
            Mode::Versus
        };
        self.team = None;
        self.arcade = None;
        self.replaying = None;
        self.desync_reported = false;
        self.sel = (c1, c2, stage);
        let saved = self.settings;
        self.settings = Settings {
            round_time: 0,
            rounds: 1,
            life: 2,
            single_round: false,
            classic_pushbox: true,
            ..saved
        };
        self.start_fight().await;
        self.settings = saved;
        if let Some(s) = self.session.as_mut() {
            s.m.rng = ume_engine::matchs::DEFAULT_SEED;
            s.seed = ume_engine::matchs::DEFAULT_SEED;
            s.net = Some(link);
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub(super) fn net_overlay(&self, n: &crate::netlink::NetLink) {
        use crate::netlink::NetStatus;
        let (text, color) = match n.status() {
            NetStatus::Connecting => (crate::i18n::tr3("CONECTANDO AO ADVERSÁRIO…", "CONNECTING TO OPPONENT…", "CONECTANDO CON EL RIVAL…").to_string(), hex("#ffcc33")),
            NetStatus::Waiting => (crate::i18n::tr3("SINCRONIZANDO…", "SYNCING…", "SINCRONIZANDO…").to_string(), hex("#ffcc33")),
            NetStatus::Lost(e) => (format!("{}: {e}", crate::i18n::tr3("CONEXÃO PERDIDA", "CONNECTION LOST", "CONEXIÓN PERDIDA")), hex("#ff4d6d")),
            NetStatus::Running => {
                if n.spectate {
                    (
                        format!(
                            "{} · {} ms · {} {}",
                            crate::i18n::tr3("ESPECTADOR", "SPECTATOR", "ESPECTADOR"),
                            n.ping_ms().unwrap_or(0),
                            n.frames_behind(),
                            crate::i18n::tr3("atrás", "behind", "atrás")
                        ),
                        hex("#a49cc0"),
                    )
                } else {
                    (
                        format!("ONLINE P{} · {} ms", n.local + 1, n.ping_ms().unwrap_or(0)),
                        hex("#a49cc0"),
                    )
                }
            }
        };
        self.ui
            .text(&text, VW / 2.0, VH - 16.0, 18.0, color, Align::Center, true);
        if n.desyncs > 0 {
            self.ui.text(
                &format!("DESSINCRONIZADO ({})", n.desyncs),
                VW / 2.0,
                VH - 40.0,
                18.0,
                hex("#ff4d6d"),
                Align::Center,
                true,
            );
        }
    }

    /// Re-simulate the last finished fight from its recorded inputs (deterministic engine → identical fight).
    pub async fn start_replay(&mut self) {
        let Some(r) = self.last_fight.clone() else {
            return;
        };
        self.mode = r.mode;
        self.team = r.team.clone();
        self.sel = r.sel;
        self.pal = r.pal;
        self.team_pal = r.team_pal.clone();
        self.replaying = Some(r);
        self.start_fight().await;
    }
}
