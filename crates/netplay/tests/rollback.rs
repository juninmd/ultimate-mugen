use ggrs::{DesyncDetection, GgrsEvent, PlayerType, SessionBuilder, SessionState};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k::*;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_zip, Vfs};
use ume_netplay::{checksum, run_sync_test, Header, MemorySocket, Netplay, Replay, Rollback};

fn kfm() -> Arc<CharDef> {
    let mut v = Vfs::new();
    add_zip(&mut v, include_bytes!("../../../assets/mugen/elecbyte.zip"), "").unwrap();
    Arc::new(load_mugen_char(&v, "chars/kfm/kfm.def", 1, false).unwrap())
}

fn new_match(d: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d.clone(), 1, 0, 2);
    m.start_round();
    m
}

const MOVES: [u32; 12] = [0, R, L, R | X, X, Y, A, B, D | X, D | R | X, U, L | D];

fn bot(seed: u64, player: usize, frame: u64) -> u32 {
    let mut x = seed ^ (player as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (frame / 6).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 31;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 29;
    MOVES[(x % MOVES.len() as u64) as usize]
}

fn script(seed: u64, frames: u64) -> Vec<[u32; 2]> {
    (0..frames).map(|f| [bot(seed, 0, f), bot(seed, 1, f)]).collect()
}

#[test]
fn snapshot_restores_exact_state() {
    let d = kfm();
    let mut m = new_match(&d);
    let inputs = script(7, 400);
    for f in &inputs[..200] {
        m.tick(f);
    }
    let snap = m.clone();
    for f in &inputs[200..] {
        m.tick(f);
    }
    let after = checksum(&m);
    m = snap;
    for f in &inputs[200..] {
        m.tick(f);
    }
    assert_eq!(checksum(&m), after);
}

#[test]
fn sync_test_detects_no_desync() {
    let d = kfm();
    let rb = run_sync_test(new_match(&d), &script(11, 900), 7).expect("sync test");
    assert!(rb.rollbacks > 0);
    assert_eq!(rb.game.game_time, new_match(&d).game_time + 900);
}

#[test]
fn replay_round_trip_reproduces_match() {
    let d = kfm();
    let mut rec = Replay::new(2);
    rec.meta.push(("p1".into(), "kfm".into()));
    let mut m = new_match(&d);
    for f in script(3, 600) {
        rec.push(&f);
        m.tick(&f);
    }
    let bytes = rec.to_bytes();
    assert!(bytes.len() < 600 * 8);
    let back = Replay::from_bytes(&bytes).unwrap();
    assert_eq!(back, rec);
    assert_eq!(back.meta("p1"), Some("kfm"));
    let mut r = new_match(&d);
    for n in 0..back.len() {
        r.tick(back.frame(n).unwrap());
    }
    assert_eq!(checksum(&r), checksum(&m));
    assert!(Replay::from_bytes(&bytes[..bytes.len() - 3]).is_err());
    assert!(Replay::from_bytes(b"nope").is_err());
}

fn sample_header(sum: Option<u128>) -> Header {
    Header {
        build: "1.2.3".into(),
        mode: "versus".into(),
        stage: "dojo".into(),
        teams: [vec!["kfm".into()], vec!["kfm".into()]],
        palettes: vec![1, 2],
        simul: false,
        seed: 0x2545_f491,
        created: 1_700_000_000,
        checksum: sum,
    }
}

#[test]
fn replay_file_of_1200_ticks_reloads_to_the_same_checksum() {
    let d = kfm();
    let mut m = new_match(&d);
    let mut rec = Replay::with_header(2, &sample_header(None));
    for f in script(11, 1200) {
        rec.push(&f);
        m.tick(&f);
    }
    let final_sum = checksum(&m);
    rec.set_header(&sample_header(Some(final_sum)));
    let bytes = rec.to_bytes();
    let back = Replay::from_bytes(&bytes).unwrap();
    assert_eq!(back.len(), 1200);
    assert_eq!(back.header().unwrap(), sample_header(Some(final_sum)));
    let mut r = new_match(&d);
    for n in 0..back.len() {
        r.tick(back.frame(n).unwrap());
    }
    assert_eq!(checksum(&r), final_sum);
    assert_eq!(Some(checksum(&r)), back.header().unwrap().checksum);
}

#[test]
fn replay_from_another_build_is_rejected_with_a_clear_message() {
    let rec = Replay::with_header(2, &sample_header(None));
    assert!(rec.check_build("1.2.3").is_ok());
    let msg = rec.check_build("2.0.0").unwrap_err();
    assert!(msg.contains("1.2.3") && msg.contains("2.0.0"), "{msg}");
    assert!(Replay::new(2).check_build("1.2.3").is_err());
    assert!(Replay::new(2).header().is_err());
}

#[test]
fn replay_header_survives_empty_teams_and_rejects_oversized_files() {
    let mut h = sample_header(None);
    h.teams = [vec![], vec!["a".into(), "b c".into()]];
    h.palettes = vec![];
    let rec = Replay::with_header(2, &h);
    assert_eq!(Replay::from_bytes(&rec.to_bytes()).unwrap().header().unwrap(), h);
    let mut huge = Replay::new(2).to_bytes();
    let at = huge.len() - 4;
    huge[at..].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Replay::from_bytes(&huge).is_err());
}

fn p2p_fight(d: &Arc<CharDef>, seed: u64, latency: u64, frames: i32) -> (u128, u128, u64, i32) {
    let (sockets, clock) = MemorySocket::network(2, latency);
    let mut peers: Vec<_> = sockets
        .into_iter()
        .enumerate()
        .map(|(me, sock)| {
            let other = 1 - me;
            let s = SessionBuilder::<Netplay<usize>>::new()
                .with_num_players(2)
                .unwrap()
                .with_max_prediction_window(12)
                .with_input_delay(2)
                .with_desync_detection_mode(DesyncDetection::On { interval: 10 })
                .add_player(PlayerType::Local, me)
                .unwrap()
                .add_player(PlayerType::Remote(other), other)
                .unwrap()
                .start_p2p_session(sock)
                .unwrap();
            (me, s, Rollback::new(new_match(d)))
        })
        .collect();
    let mut steps = 0u64;
    while peers.iter().any(|(_, s, _)| s.confirmed_frame() < frames) {
        steps += 1;
        assert!(steps < 200_000, "session stalled");
        *clock.borrow_mut() += 1;
        for (me, s, rb) in peers.iter_mut() {
            s.poll_remote_clients();
            for e in s.events() {
                if let GgrsEvent::DesyncDetected { frame, .. } = e {
                    panic!("desync at frame {frame} (seed {seed})");
                }
            }
            if s.current_state() != SessionState::Running {
                continue;
            }
            let frame = s.current_frame().max(0) as u64;
            s.add_local_input(*me, bot(seed, *me, frame)).unwrap();
            match s.advance_frame() {
                Ok(r) => {
                    rb.handle(r);
                }
                Err(ggrs::GgrsError::PredictionThreshold) => {}
                Err(e) => panic!("{e}"),
            }
        }
    }
    let at = |i: usize| peers[i].2.checksum_at(frames).expect("checksum of the final frame");
    (at(0), at(1), peers[0].2.rollbacks + peers[1].2.rollbacks, peers[0].2.game.round_no)
}

#[test]
fn p2p_fights_at_100ms_stay_in_sync() {
    let d = kfm();
    let fights: u64 = std::env::var("UME_NETPLAY_FIGHTS").ok().and_then(|v| v.parse().ok()).unwrap_or(50);
    let mut rollbacks = 0;
    for seed in 0..fights {
        let (a, b, r, _) = p2p_fight(&d, seed, 6, 2400);
        assert_eq!(a, b, "fight {seed} ended in different states");
        rollbacks += r;
    }
    assert!(rollbacks > 0, "latency should force rollbacks");
}

#[test]
fn bots_actually_fight() {
    let d = kfm();
    let mut m = new_match(&d);
    for f in script(5, 2400) {
        m.tick(&f);
    }
    assert!(m.fighters.iter().any(|f| f.life < f.life_max) || m.round_no > 1);
}

#[test]
fn udp_localhost_fight_stays_in_sync() {
    use ggrs::UdpNonBlockingSocket;
    use std::net::SocketAddr;
    let d = kfm();
    let ports = [47311u16, 47312];
    let addr = |p: u16| -> SocketAddr { format!("127.0.0.1:{p}").parse().unwrap() };
    let mut peers: Vec<_> = (0..2)
        .map(|me| {
            let other = 1 - me;
            let s = SessionBuilder::<Netplay<SocketAddr>>::new()
                .with_num_players(2)
                .unwrap()
                .with_input_delay(2)
                .with_desync_detection_mode(DesyncDetection::On { interval: 10 })
                .add_player(PlayerType::Local, me)
                .unwrap()
                .add_player(PlayerType::Remote(addr(ports[other])), other)
                .unwrap()
                .start_p2p_session(UdpNonBlockingSocket::bind_to_port(ports[me]).unwrap())
                .unwrap();
            (me, s, Rollback::new(new_match(&d)))
        })
        .collect();
    let target = 600;
    let start = std::time::Instant::now();
    while peers.iter().any(|(_, s, _)| s.confirmed_frame() < target) {
        assert!(start.elapsed().as_secs() < 60, "udp session stalled");
        for (me, s, rb) in peers.iter_mut() {
            s.poll_remote_clients();
            for e in s.events() {
                assert!(!matches!(e, GgrsEvent::DesyncDetected { .. }), "desync over udp");
            }
            if s.current_state() != SessionState::Running {
                continue;
            }
            let frame = s.current_frame().max(0) as u64;
            s.add_local_input(*me, bot(9, *me, frame)).unwrap();
            if let Ok(r) = s.advance_frame() {
                rb.handle(r);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(peers[0].2.checksum_at(target), peers[1].2.checksum_at(target));
}
