with open('crates/engine/src/native/compile.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
    'use crate::chardef::{default_constants, AnimFrame, Animation, Blend, CharDef, MoveInfo, Source, Sprite};',
    'use crate::chardef::{default_constants, AnimFrame, Animation, Blend, CharDef, ImageData, MoveInfo, Source, Sprite};'
)

with open('crates/engine/src/native/compile.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('Added ImageData to imports')
