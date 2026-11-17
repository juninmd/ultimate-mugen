//! Replay share codes: one copy-paste line per replay, decoded with the same checks as a saved file.
use crate::replays::{StoredReplay, MAX_REPLAY_BYTES};

const PREFIX: &str = "umr1.";
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn encode(replay: &StoredReplay) -> String {
    format!("{PREFIX}{}", base64url(replay.to_json().as_bytes()))
}

pub fn decode(code: &str) -> Result<StoredReplay, String> {
    let body = code
        .trim()
        .strip_prefix(PREFIX)
        .ok_or("Código inválido: prefixo desconhecido")?;
    if body.len() > MAX_REPLAY_BYTES / 3 * 4 + 4 {
        return Err("Código inválido: replay grande demais".into());
    }
    let bytes = unbase64url(body)?;
    let text = String::from_utf8(bytes).map_err(|_| "Código inválido: texto corrompido")?;
    StoredReplay::from_json(&text)
}

fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, &b)| acc | (u32::from(b) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

fn unbase64url(text: &str) -> Result<Vec<u8>, String> {
    if text.len() % 4 == 1 {
        return Err("Código inválido: tamanho incorreto".into());
    }
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    for chunk in text.as_bytes().chunks(4) {
        let mut n = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            n |= u32::from(symbol_value(c)?) << (18 - 6 * i);
        }
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..chunk.len() - 1]);
    }
    Ok(out)
}

fn symbol_value(c: u8) -> Result<u8, String> {
    ALPHABET
        .iter()
        .position(|&a| a == c)
        .map(|p| p as u8)
        .ok_or_else(|| "Código inválido: caractere fora do alfabeto".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replays::StoredRules;

    fn sample() -> StoredReplay {
        StoredReplay {
            label: "Partida de treino".into(),
            saved_at: 1_700_000_000,
            mode: "Versus".into(),
            chars: ["umf:kfm".into(), "umf:kfm".into()],
            stage: "dojo".into(),
            palettes: [1, 2],
            member_palettes: vec![],
            seed: 42,
            life: [0.5, 1.0],
            teams: None,
            simul: false,
            tag: false,
            rules: StoredRules {
                rounds_to_win: 2,
                ..Default::default()
            },
            netplay: None,
            runs: vec![(2, vec![0, 4]), (3, vec![8, 0])],
        }
    }

    #[test]
    fn a_replay_survives_the_code_round_trip() {
        let replay = sample();
        let code = encode(&replay);
        assert!(code.starts_with(PREFIX));
        assert_eq!(decode(&code).unwrap(), replay);
    }

    #[test]
    fn every_length_modulo_three_round_trips() {
        for len in 0..8 {
            let bytes: Vec<u8> = (0..len as u8).map(|b| b.wrapping_mul(37)).collect();
            assert_eq!(unbase64url(&base64url(&bytes)).unwrap(), bytes, "len {len}");
        }
    }

    #[test]
    fn rejects_foreign_prefixes_bad_symbols_and_truncated_codes() {
        assert!(decode("abc").is_err());
        assert!(decode("umr1.a*bc").is_err());
        assert!(decode("umr1.a").is_err());
        assert!(decode(&format!("{PREFIX}{}", "A".repeat(MAX_REPLAY_BYTES))).is_err());
    }
}
