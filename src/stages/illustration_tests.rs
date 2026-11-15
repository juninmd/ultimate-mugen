use super::*;

#[test]
fn thumbnails_are_small_and_cached_per_stage() {
    for id in ILLUSTRATION_IDS.iter().chain(wonders::WONDER_IDS.iter()) {
        let image = illustration_preview(id).unwrap();
        assert_eq!((image.w, image.h), (240, 135));
        assert!(Arc::ptr_eq(&image, &illustration_preview(id).unwrap()));
    }
    assert!(!Arc::ptr_eq(
        &illustration_preview("dojo").unwrap(),
        &illustration_preview("neon").unwrap()
    ));
    assert!(illustration_preview("unknown").is_none());
}

#[test]
fn invalid_stage_image_is_an_error() {
    assert!(decode_illustration(b"invalid image").is_err());
}
