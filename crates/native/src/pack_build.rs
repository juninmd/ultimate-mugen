const MAGIC: &[u8; 4] = b"UMEP";
const VERSION: u8 = 2;

pub fn encode_pack(character: &str, files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let id_len = u16::try_from(character.len()).map_err(|_| "sprite pack: id too long".to_string())?;
    let mut out = Vec::with_capacity(64 + files.iter().map(|(name, data)| name.len() + data.len() + 6).sum::<usize>());
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&id_len.to_le_bytes());
    out.extend_from_slice(character.as_bytes());
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (name, data) in files {
        let name_len = u16::try_from(name.len()).map_err(|_| format!("sprite pack: name too long: {name}"))?;
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::SpritePack;

    #[test]
    fn encoded_pack_parses_back() {
        let files = vec![("a.png".to_string(), vec![1, 2, 3]), ("meta.json".to_string(), b"{}".to_vec())];
        let bytes = encode_pack("hero", &files).unwrap();
        let pack = SpritePack::parse(&bytes).unwrap();
        assert_eq!(pack.character, "hero");
    }

    #[test]
    fn full_size_fighters_above_sixteen_mib_still_load() {
        let big = vec![7u8; 21 << 20];
        let bytes = encode_pack("goku_sized", &[("a.png".to_string(), big)]).unwrap();
        assert!(SpritePack::parse(&bytes).is_ok());
        let hostile = vec![0u8; crate::pack::MAX_PACK_BYTES + 1];
        assert!(SpritePack::parse(&hostile).is_err());
    }
}
