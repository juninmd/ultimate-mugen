use super::types::NativeChar;

impl NativeChar {
    pub fn with_sprite_animation_defaults(&self) -> Self {
        super::compile::with_defaults(self)
    }

    pub fn sprite_dir(&self) -> &str {
        self.sprite_set.as_deref().unwrap_or(&self.id)
    }
}
