use ggrs::{DesyncDetection, GgrsError, GgrsEvent, P2PSession, PlayerType, SessionBuilder, SessionState, UdpNonBlockingSocket};
use std::net::SocketAddr;
use ume_engine::matchs::Match;
use ume_netplay::{drive, Netplay};

pub type Net = Netplay<SocketAddr>;

#[derive(Clone, Debug, PartialEq)]
pub struct NetArgs {
    pub side: usize,
    pub port: u16,
    pub peer: SocketAddr,
    pub delay: usize,
    pub chars: Option<(String, String)>,
    pub stage: Option<String>,
}

impl NetArgs {
    pub fn parse(args: &[String]) -> Result<Option<NetArgs>, String> {
        let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let Some(side) = get("--netplay") else { return Ok(None) };
        let side = match side.as_str() {
            "1" => 0,
            "2" => 1,
            other => return Err(format!("--netplay expects 1 or 2, got {other}")),
        };
        let peer = get("--peer").ok_or("--netplay needs --peer <ip:port>")?.parse().map_err(|e| format!("--peer: {e}"))?;
        let port = get("--port").map(|p| p.parse().map_err(|e| format!("--port: {e}"))).transpose()?.unwrap_or(7000 + side as u16);
        let delay = get("--delay").map(|p| p.parse().map_err(|e| format!("--delay: {e}"))).transpose()?.unwrap_or(2);
        let chars = get("--chars").and_then(|c| c.split_once(',').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())));
        Ok(Some(NetArgs { side, port, peer, delay, chars, stage: get("--stage") }))
    }
}

pub enum NetStatus {
    Connecting,
    Running,
    Waiting,
    Lost(String),
}

pub struct NetLink {
    session: P2PSession<Net>,
    pub local: usize,
    skip: u32,
    pub rollbacks: u64,
    pub desyncs: u32,
    pub lost: Option<String>,
}

impl NetLink {
    pub fn connect(a: &NetArgs) -> Result<NetLink, String> {
        let socket = UdpNonBlockingSocket::bind_to_port(a.port).map_err(|e| format!("port {}: {e}", a.port))?;
        let remote = 1 - a.side;
        let session = SessionBuilder::<Net>::new()
            .with_num_players(2)
            .and_then(|b| b.add_player(PlayerType::Local, a.side))
            .and_then(|b| b.add_player(PlayerType::Remote(a.peer), remote))
            .map_err(|e| e.to_string())?
            .with_input_delay(a.delay)
            .with_max_prediction_window(12)
            .with_desync_detection_mode(DesyncDetection::On { interval: 60 })
            .start_p2p_session(socket)
            .map_err(|e| e.to_string())?;
        Ok(NetLink { session, local: a.side, skip: 0, rollbacks: 0, desyncs: 0, lost: None })
    }

    pub fn status(&self) -> NetStatus {
        match (&self.lost, self.session.current_state()) {
            (Some(e), _) => NetStatus::Lost(e.clone()),
            (None, SessionState::Running) if self.skip > 0 => NetStatus::Waiting,
            (None, SessionState::Running) => NetStatus::Running,
            _ => NetStatus::Connecting,
        }
    }

    pub fn ping_ms(&self) -> Option<u128> {
        self.session.network_stats(1 - self.local).ok().map(|s| s.ping)
    }

    pub fn step<F: FnMut(&mut Match, &[u32], bool)>(&mut self, game: &mut Match, local_bits: u32, tick: F) -> bool {
        self.session.poll_remote_clients();
        for e in self.session.events() {
            match e {
                GgrsEvent::WaitRecommendation { skip_frames } => self.skip += skip_frames,
                GgrsEvent::Disconnected { .. } => self.lost = Some("opponent disconnected".into()),
                GgrsEvent::DesyncDetected { .. } => self.desyncs += 1,
                _ => {}
            }
        }
        if self.lost.is_some() || self.session.current_state() != SessionState::Running {
            return false;
        }
        if self.skip > 0 {
            self.skip -= 1;
            return false;
        }
        if let Err(e) = self.session.add_local_input(self.local, local_bits) {
            self.lost = Some(e.to_string());
            return false;
        }
        match self.session.advance_frame() {
            Ok(requests) => {
                self.rollbacks += drive(game, requests, tick, |_, _| {}).rollbacks;
                true
            }
            Err(GgrsError::PredictionThreshold) => false,
            Err(e) => {
                self.lost = Some(e.to_string());
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        std::iter::once("ultimate-mugen").chain(s.split_whitespace()).map(String::from).collect()
    }

    #[test]
    fn parses_netplay_flags() {
        assert_eq!(NetArgs::parse(&args("--res 1280x720")).unwrap(), None);
        let a = NetArgs::parse(&args("--netplay 2 --peer 127.0.0.1:7000 --chars kfm,kfm --stage dojo --delay 3")).unwrap().unwrap();
        assert_eq!((a.side, a.port, a.delay), (1, 7001, 3));
        assert_eq!(a.chars, Some(("kfm".into(), "kfm".into())));
        assert_eq!(a.stage.as_deref(), Some("dojo"));
        assert!(NetArgs::parse(&args("--netplay 3 --peer 127.0.0.1:1")).is_err());
        assert!(NetArgs::parse(&args("--netplay 1")).is_err());
    }
}
