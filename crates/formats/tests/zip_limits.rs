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

#[test]
fn accepts_deflate_with_exaggerated_size_hint() {
    let deflated = miniz_oxide::deflate::compress_to_vec(b"hello", 1);
    let zip = stored_zip_with_deflate(256 << 20, &deflated);
    let mut v = Vfs::new();
    assert_eq!(add_zip(&mut v, &zip, "").unwrap(), 1);
    assert_eq!(v.values().next().unwrap(), b"hello");
}

#[test]
fn rejects_oversized_stored_entry_before_reading_data() {
    let mut zip = stored_zip_with_deflate(1, b"x");
    let central = zip.windows(4).position(|s| s == 0x0201_4b50u32.to_le_bytes()).unwrap();
    zip[central + 10..central + 12].copy_from_slice(&0u16.to_le_bytes());
    zip[central + 20..central + 24].copy_from_slice(&(257u32 << 20).to_le_bytes());
    let mut v = Vfs::new();
    assert!(add_zip(&mut v, &zip, "").unwrap_err().contains("entry too large"));
}
