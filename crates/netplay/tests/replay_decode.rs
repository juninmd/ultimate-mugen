use ume_netplay::Replay;

fn header(players: u32, frames: u32) -> Vec<u8> {
    let mut bytes = b"UMRP".to_vec();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&players.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&frames.to_le_bytes());
    bytes
}

#[test]
fn rejects_runs_that_exceed_the_declared_frame_count() {
    let mut bytes = header(1, 1);
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&7u32.to_le_bytes());

    assert!(Replay::from_bytes(&bytes).is_err());
}

#[test]
fn rejects_bytes_after_the_declared_frame_stream() {
    let mut bytes = Replay::new(1);
    bytes.push(&[7]);
    let mut encoded = bytes.to_bytes().unwrap();
    encoded.push(0xff);

    assert!(Replay::from_bytes(&encoded).is_err());
}

#[test]
fn rejects_frame_expansion_above_the_decoded_size_limit() {
    let mut bytes = header(64, 262_145);
    bytes.extend_from_slice(&262_145u32.to_le_bytes());
    bytes.extend((0..64).flat_map(|_| 0u32.to_le_bytes()));

    assert!(Replay::from_bytes(&bytes).is_err());
}

#[test]
fn rejects_excessive_metadata_entries() {
    let mut bytes = b"UMRP".to_vec();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&16_385u32.to_le_bytes());
    for _ in 0..16_385 {
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());

    assert!(Replay::from_bytes(&bytes).is_err());
}

#[test]
fn out_of_range_frame_index_does_not_overflow() {
    assert!(Replay::new(2).frame(usize::MAX).is_none());
}

#[test]
fn writer_output_at_the_decoded_limit_remains_readable() {
    let replay = Replay { players: 1, meta: Vec::new(), frames: (0..16_777_216u32).collect() };
    let encoded = replay.to_bytes().unwrap();

    assert!(encoded.len() > 64 * 1024 * 1024);
    assert_eq!(Replay::from_bytes(&encoded).unwrap(), replay);
}
