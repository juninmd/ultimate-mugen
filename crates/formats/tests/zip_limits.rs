use ume_formats::vfs::{add_zip, Vfs};

fn stored_zip_with_deflate(declared_size: u32, deflated: &[u8]) -> Vec<u8> {
    let name = b"a.bin";
    let mut z = Vec::new();
    z.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
    z.extend_from_slice(&[20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
    z.extend_from_slice(&0u32.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes());
    z.extend_from_slice(&declared_size.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z.extend_from_slice(name);
    z.extend_from_slice(deflated);
    let cd_start = z.len() as u32;
    z.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
    z.extend_from_slice(&[20, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
    z.extend_from_slice(&0u32.to_le_bytes());
    z.extend_from_slice(&(deflated.len() as u32).to_le_bytes());
    z.extend_from_slice(&declared_size.to_le_bytes());
    z.extend_from_slice(&(name.len() as u16).to_le_bytes());
    z.extend_from_slice(&[0; 12]);
    z.extend_from_slice(&0u32.to_le_bytes());
    z.extend_from_slice(&0u32.to_le_bytes());
    z.extend_from_slice(name);
    let cd_size = z.len() as u32 - cd_start;
    z.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    z.extend_from_slice(&[0; 4]);
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&1u16.to_le_bytes());
    z.extend_from_slice(&cd_size.to_le_bytes());
    z.extend_from_slice(&cd_start.to_le_bytes());
    z.extend_from_slice(&0u16.to_le_bytes());
    z
}

#[test]
fn rejects_entry_inflating_past_limit() {
    let payload = vec![0u8; 300 << 20];
    let deflated = miniz_oxide::deflate::compress_to_vec(&payload, 1);
    drop(payload);
    let zip = stored_zip_with_deflate(u32::MAX, &deflated);
    let mut v = Vfs::new();
    assert!(add_zip(&mut v, &zip, "").is_err());
}

#[test]
fn accepts_small_deflated_entry() {
    let deflated = miniz_oxide::deflate::compress_to_vec(b"hello", 1);
    let zip = stored_zip_with_deflate(5, &deflated);
    let mut v = Vfs::new();
    assert_eq!(add_zip(&mut v, &zip, "").unwrap(), 1);
}
