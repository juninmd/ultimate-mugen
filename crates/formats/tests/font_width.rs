use ume_formats::fnt::decode_fnt;

#[test]
fn text_width_saturates_for_extreme_font_metrics() {
    let font =
        decode_fnt(b"[Def]\nType = Bitmap\nSize = 2147483647,8\nSpacing = 2147483647,0\n").unwrap();
    assert_eq!(font.text_width("AAA"), i32::MAX);
}
