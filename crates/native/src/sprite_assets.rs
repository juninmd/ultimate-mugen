use super::pixel::sprite_roots;

pub fn sprite_assets_found(id: &str) -> bool {
    super::embedded::has(id) || sprite_roots(id).iter().any(|root| root.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_repository_sprites_and_rejects_unknown_fighters() {
        assert!(sprite_assets_found("kaito"));
        assert!(!sprite_assets_found("no-such-fighter-xyz"));
    }
}
