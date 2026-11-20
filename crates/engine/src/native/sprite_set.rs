use super::types::NativeChar;

impl NativeChar {
    pub fn with_sprite_animation_defaults(&self) -> Self {
        super::compile::with_defaults(self)
    }
}
