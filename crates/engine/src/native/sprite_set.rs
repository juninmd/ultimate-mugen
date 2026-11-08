use super::types::NativeChar;

impl NativeChar {
    pub fn sprite_dir(&self) -> &str {
        self.sprite_set.as_deref().unwrap_or(&self.id)
    }
}
