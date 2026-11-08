pub fn fresh_seed() -> u32 {
    #[cfg(target_arch = "wasm32")]
    let t = (ume_shell::quad::get_time() * 1_000_000.0) as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let x = (t ^ t.rotate_left(29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    ((x >> 32) as u32).max(1)
}
