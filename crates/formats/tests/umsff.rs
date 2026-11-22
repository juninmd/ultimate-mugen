use std::sync::Arc;
use ume_formats::sff::{decode_sff, DecodedSff, DecodedSprite, Palette};
use ume_formats::umsff::{decode_umsff, encode_umsff, is_umsff};

fn palette(seed: u8) -> Arc<Vec<u8>> {
    Arc::new(
        (0..1024u32)
            .map(|i| (i as u8).wrapping_mul(seed).wrapping_add(i as u8 / 4))
            .collect(),
    )
}

fn indexed(
    group: i32,
    index: i32,
    pixels: &Arc<Vec<u8>>,
    palette: &Arc<Vec<u8>>,
    slot: i32,
) -> DecodedSprite {
    DecodedSprite {
        group,
        index,
        axis_x: 3,
        axis_y: -4,
        width: 8,
        height: 6,
        indexed: Some(pixels.clone()),
        rgba: None,
        palette: Some(palette.clone()),
        palette_index: slot,
    }
}

fn sample() -> DecodedSff {
    let (first, second) = (palette(3), palette(5));
    let body = Arc::new(
        (0..48u8)
            .map(|i| if i % 7 == 0 { 0 } else { i })
            .collect::<Vec<u8>>(),
    );
    let other = Arc::new(vec![0u8; 48]);
    let color = Arc::new(
        (0..8 * 6 * 4)
            .map(|i| if i % 4 == 3 { 255 } else { i as u8 })
            .collect::<Vec<u8>>(),
    );
    let mut true_color = indexed(9000, 1, &body, &first, -1);
    true_color.indexed = None;
    true_color.rgba = Some(color);
    true_color.palette = None;
    let empty = DecodedSprite {
        group: 5,
        index: 5,
        axis_x: 0,
        axis_y: 0,
        width: 0,
        height: 0,
        indexed: None,
        rgba: None,
        palette: None,
        palette_index: -1,
    };
    DecodedSff {
        version: 2,
        sprites: vec![
            indexed(0, 0, &body, &first, 0),
            indexed(0, 1, &body, &second, 1),
            indexed(1, 0, &other, &first, 0),
            true_color,
            empty,
        ],
        palettes: vec![
            Palette {
                group: 1,
                index: 1,
                data: first,
            },
            Palette {
                group: 1,
                index: 2,
                data: second,
            },
        ],
    }
}

fn same(left: &DecodedSprite, right: &DecodedSprite) {
    assert_eq!(
        (left.group, left.index, left.axis_x, left.axis_y),
        (right.group, right.index, right.axis_x, right.axis_y)
    );
    assert_eq!(
        (left.width, left.height, left.palette_index),
        (right.width, right.height, right.palette_index)
    );
    assert_eq!(left.indexed.as_deref(), right.indexed.as_deref());
    assert_eq!(left.rgba.as_deref(), right.rgba.as_deref());
    assert_eq!(left.palette.as_deref(), right.palette.as_deref());
}

#[test]
fn cooked_file_decodes_to_the_same_sprites() {
    let source = sample();
    let bytes = encode_umsff(&source).unwrap();
    assert!(is_umsff(&bytes));
    let back = decode_sff(&bytes).unwrap();
    assert_eq!(back.version, 2);
    assert_eq!(back.sprites.len(), source.sprites.len());
    for (left, right) in source.sprites.iter().zip(&back.sprites) {
        same(left, right);
    }
    for (left, right) in source.palettes.iter().zip(&back.palettes) {
        assert_eq!(
            (left.group, left.index, &*left.data),
            (right.group, right.index, &*right.data)
        );
    }
}

#[test]
fn shared_pixels_stay_shared_after_decoding() {
    let back = decode_sff(&encode_umsff(&sample()).unwrap()).unwrap();
    assert!(Arc::ptr_eq(
        back.sprites[0].indexed.as_ref().unwrap(),
        back.sprites[1].indexed.as_ref().unwrap()
    ));
    assert!(!Arc::ptr_eq(
        back.sprites[0].indexed.as_ref().unwrap(),
        back.sprites[2].indexed.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        back.sprites[0].palette.as_ref().unwrap(),
        &back.palettes[0].data
    ));
}

#[test]
fn wanted_filter_keeps_only_the_asked_sprites() {
    let bytes = encode_umsff(&sample()).unwrap();
    let back = decode_umsff(&bytes, |group, index| (group, index) == (1, 0)).unwrap();
    assert!(back.sprites[2].indexed.is_some());
    for at in [0, 1, 3] {
        assert!(
            back.sprites[at].indexed.is_none()
                && back.sprites[at].rgba.is_none()
                && back.sprites[at].width == 0
        );
    }
    assert_eq!(
        (
            back.sprites[0].group,
            back.sprites[0].axis_x,
            back.sprites[0].axis_y
        ),
        (0, 3, -4)
    );
}

#[test]
fn mismatched_sprite_size_is_rejected_when_cooking() {
    let mut broken = sample();
    broken.sprites[0].width = 9;
    assert!(encode_umsff(&broken).is_err());
}

#[test]
fn corrupt_files_fail_without_panicking() {
    let bytes = encode_umsff(&sample()).unwrap();
    for cut in (0..bytes.len()).step_by(7) {
        let _ = decode_sff(&bytes[..cut]);
    }
    let mut bad_version = bytes.clone();
    bad_version[4] = 99;
    assert!(decode_sff(&bad_version).is_err());
    let mut huge = bytes.clone();
    huge[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(decode_sff(&huge).is_err());
    for at in 8..bytes.len().min(400) {
        let mut flipped = bytes.clone();
        flipped[at] ^= 0xff;
        let _ = decode_sff(&flipped);
    }
}

#[test]
fn declared_sizes_over_the_budget_are_rejected() {
    let mut bytes = encode_umsff(&sample()).unwrap();
    let rows_at = bytes.len() - 5 * 36;
    for row in 0..5 {
        let at = rows_at + row * 36 + 16;
        bytes[at..at + 4].copy_from_slice(&8192u32.to_le_bytes());
        bytes[at + 4..at + 8].copy_from_slice(&8192u32.to_le_bytes());
    }
    assert!(decode_sff(&bytes).is_err());
}
