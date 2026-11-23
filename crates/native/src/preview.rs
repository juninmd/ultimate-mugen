use super::compile::compile_native_with;
use super::types::NativeChar;
use ume_engine::chardef::CharDef;

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

pub fn menu_sprite_files(nc: &NativeChar) -> Vec<String> {
    super::pack::trace_reads(|| {
        super::portrait::select_portrait(nc, 1);
        compile_native_preview(nc, 1);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn the_menu_preview_does_not_compile_the_form_target_in_full() {
        let suzume = crate::parse_char(include_str!("../../../assets/chars/suzume.json")).unwrap();
        let oboro = crate::parse_char(include_str!("../../../assets/chars/oboro.json")).unwrap();
        crate::register_native(Arc::new(oboro));
        let preview = compile_native_preview(&suzume, 1);
        let target = preview.forms.iter().find_map(|form| form.target.as_ref()).expect("suzume transforms into oboro");
        let full = compile_native_with(&suzume, 1, false, None);
        let full_target = full.forms.iter().find_map(|form| form.target.as_ref()).unwrap();
        assert!(target.anims.keys().all(|no| MENU_ANIMS.contains(no)), "only the menu animations");
        assert!(full_target.anims.len() > target.anims.len());
    }
}
