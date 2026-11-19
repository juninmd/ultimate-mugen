pub fn with_sprite_animation_defaults(
    sprite_set: crate::native::SpriteSet,
    animation: Option<crate::native::Animation>,
) -> crate::native::SpriteSet {
    let mut sprite_set = sprite_set;

    // Apply default animation settings if not provided
    if let Some(animation) = animation {
        sprite_set.set_animation(animation);
    }

    sprite_set
}