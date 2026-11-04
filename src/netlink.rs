use ggrs::{DesyncDetection, GgrsError, GgrsEvent, P2PSession, PlayerType, SessionBuilder, SessionState, SpectatorSession, UdpNonBlockingSocket};
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
    /// third connection type: receive confirmed inputs and render without sending its own
    pub spectate: bool,
}

impl NetArgs {
    pub fn parse(args: &[String]) -> Result<Option<NetArgs>, String> {
        let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let has = |k: &str| args.iter().any(|a| a == k);
        let spectate = has("--spectate");
        let Some(side) = get("--netplay") else {
            if !spectate { return Ok(None); }
            let peer = get("--peer").ok_or("--spectate needs --peer <host:port>")?.parse().map_err(|e| format!("--peer: {e}"))?;
            let port = get("--port").map(|p| p.parse().map_err(|e| format!("--port: {e}"))).transpose()?.unwrap_or(7002);
            let delay = get("--delay").map(|p| p.parse().map_err(|e| format!("--delay: {e}"))).transpose()?.unwrap_or(2);
            let chars = get("--chars").and_then(|c| c.split_once(',').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())));
            return Ok(Some(NetArgs { side: 0, port, peer, delay, chars, stage: get("--stage"), spectate: true }));
        };
        let side = match side.as_str() {
            "1" => 0,
            "2" => 1,
            other => return Err(format!("--netplay expects 1 or 2, got {other}")),
        };
        let peer = get("--peer").ok_or("--netplay needs --peer <ip:port>")?.parse().map_err(|e| format!("--peer: {e}"))?;
        let port = get("--port").map(|p| p.parse().map_err(|e| format!("--port: {e}"))).transpose()?.unwrap_or(7000 + side as u16);
        let delay = get("--delay").map(|p| p.parse().map_err(|e| format!("--delay: {e}"))).transpose()?.unwrap_or(2);
        let chars = get("--chars").and_then(|c| c.split_once(',').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())));
        Ok(Some(NetArgs { side, port, peer, delay, chars, stage: get("--stage"), spectate }))
    }
}

pub enum NetStatus {
    Connecting,
    Running,
    Waiting,
    Lost(String),
}

enum Link {
    Player(P2PSession<Net>),
    Spectator(SpectatorSession<Net>),
}

pub struct NetLink {
    link: Link,
    pub local: usize,
    pub spectate: bool,
    pub config: crate::replays::NetplayConfig,
    skip: u32,
    pub rollbacks: u64,
    pub desyncs: u32,
    pub lost: Option<String>,
}

pub const ROLLBACK_WINDOW: i32 = 12;

impl NetLink {
    pub fn connect(a: &NetArgs) -> Result<NetLink, String> {
        let socket = UdpNonBlockingSocket::bind_to_port(a.port).map_err(|e| format!("port {}: {e}", a.port))?;
        let config = crate::replays::NetplayConfig { side: if a.spectate { -1 } else { a.side as i32 }, peer: a.peer.to_string(), port: a.port, delay: a.delay, rollback_window: ROLLBACK_WINDOW, players: 2, spectate: a.spectate };
        if a.spectate {
            let session = SessionBuilder::<Net>::new()
                .with_num_players(2)
                .map_err(|e| e.to_string())?
                .with_input_delay(a.delay)
                .start_spectator_session(a.peer, socket);
            return Ok(NetLink { link: Link::Spectator(session), local: 0, spectate: true, config, skip: 0, rollbacks: 0, desyncs: 0, lost: None });
        }
        let remote = 1 - a.side;
        let session = SessionBuilder::<Net>::new()
            .with_num_players(2)
            .and_then(|b| b.add_player(PlayerType::Local, a.side))
            .and_then(|b| b.add_player(PlayerType::Remote(a.peer), remote))
            .map_err(|e| e.to_string())?
            .with_input_delay(a.delay)
            .with_max_prediction_window(ROLLBACK_WINDOW as usize)
            .with_desync_detection_mode(DesyncDetection::On { interval: 60 })
            .start_p2p_session(socket)
            .map_err(|e| e.to_string())?;
        Ok(NetLink { link: Link::Player(session), local: a.side, spectate: false, config, skip: 0, rollbacks: 0, desyncs: 0, lost: None })
    }

    fn state(&self) -> SessionState {
        match &self.link { Link::Player(s) => s.current_state(), Link::Spectator(s) => s.current_state() }
    }

    pub fn status(&self) -> NetStatus {
        match (&self.lost, self.state()) {
            (Some(e), _) => NetStatus::Lost(e.clone()),
            (None, SessionState::Running) if self.skip > 0 => NetStatus::Waiting,
            (None, SessionState::Running) => NetStatus::Running,
            _ => NetStatus::Connecting,
        }
    }

    pub fn ping_ms(&self) -> Option<u128> {
        match &self.link {
            Link::Player(s) => s.network_stats(1 - self.local).ok().map(|s| s.ping),
            Link::Spectator(s) => s.network_stats().ok().map(|s| s.ping),
        }
    }

    /// Faults shared by both connection types (disconnects and desyncs).
    fn faults(&mut self, events: impl Iterator<Item = GgrsEvent<Net>>) {
        for e in events {
            match e {
                GgrsEvent::Disconnected { .. } => self.lost = Some("opponent disconnected".into()),
                GgrsEvent::DesyncDetected { .. } => self.desyncs += 1,
                _ => {}
            }
        }
    }

    pub fn step<F: FnMut(&mut Match, &[u32], bool)>(&mut self, game: &mut Match, local_bits: u32, tick: F) -> bool {
        let events: Vec<GgrsEvent<Net>> = match &mut self.link {
            Link::Spectator(session) => { session.poll_remote_clients(); session.events().collect() }
            Link::Player(session) => { session.poll_remote_clients(); session.events().collect() }
        };
        for e in events {
            match e {
                GgrsEvent::WaitRecommendation { skip_frames } => self.skip += skip_frames,
                GgrsEvent::Disconnected { .. } => self.lost = Some("opponent disconnected".into()),
                GgrsEvent::DesyncDetected { .. } => self.desyncs += 1,
                _ => {}
            }
        }
        if self.lost.is_some() { return false; }
        match &mut self.link {
            Link::Spectator(session) => {
                if session.current_state() != SessionState::Running { return false; }
                match session.advance_frame() {
                    Ok(requests) => { self.rollbacks += drive(game, requests, tick, |_, _| {}).rollbacks; true }
                    Err(GgrsError::PredictionThreshold) => false,
                    Err(e) => { self.lost = Some(e.to_string()); false }
                }
            }
            Link::Player(session) => {
                if session.current_state() != SessionState::Running { return false; }
                if self.skip > 0 { self.skip -= 1; return false; }
                if let Err(e) = session.add_local_input(self.local, local_bits) { self.lost = Some(e.to_string()); return false; }
                match session.advance_frame() {
                    Ok(requests) => { self.rollbacks += drive(game, requests, tick, |_, _| {}).rollbacks; true }
                    Err(GgrsError::PredictionThreshold) => false,
                    Err(e) => { self.lost = Some(e.to_string()); false }
                }
            }
        }
    }

    /// Spectator sessions can fall behind the host; surfaced in the overlay.
    pub fn frames_behind(&self) -> usize { match &self.link { Link::Spectator(s) => s.frames_behind_host(), _ => 0 } }
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
        assert!(!a.spectate);
        assert!(NetArgs::parse(&args("--netplay 3 --peer 127.0.0.1:1")).is_err());
        assert!(NetArgs::parse(&args("--netplay 1")).is_err());
    }

    #[test]
    fn parses_a_spectator_connection() {
        let a = NetArgs::parse(&args("--spectate --peer 127.0.0.1:7000 --delay 4")).unwrap().unwrap();
        assert!(a.spectate);
        assert_eq!((a.side, a.delay, a.port), (0, 4, 7002));
        assert_eq!(a.port, 7002, "the spectator binds its own port");
        assert!(NetArgs::parse(&args("--spectate")).is_err(), "--spectate needs --peer");
    }
}
