use super::compile::compile_native_with;
use super::types::NativeChar;
use crate::chardef::CharDef;

const MENU_ANIMS: [i32; 2] = [0, 181];

pub fn compile_native_preview(nc: &NativeChar, pal_no: i32) -> CharDef {
    if cfg!(target_arch = "wasm32") && !super::pack::contains(&nc.id) {
        let mut slim = nc.clone();
        for (_, anim) in slim.anims.0.iter_mut() {
            anim.frames.truncate(1);
            anim.loop_start = None;
        }
        return compile_native_with(&slim, pal_no, true, Some(&MENU_ANIMS));
    }
    compile_native_with(nc, pal_no, true, Some(&MENU_ANIMS))
}
