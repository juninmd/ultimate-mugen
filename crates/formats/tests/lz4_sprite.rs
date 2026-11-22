use ume_formats::lz4_sprite::{
    decode, decode_rgba, encode_indexed, encode_rgba, is_lz4_sprite, Pixels, MAX_SIDE,
};

fn noise(len: usize, seed: u32) -> Vec<u8> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (state >> 24) as u8
        })
        .collect()
}

fn framed(width: usize, height: usize, margin: usize, channels: usize) -> Vec<u8> {
    let mut data = vec![0u8; width * height * channels];
    for row in margin..height - margin {
        for col in margin..width - margin {
            let at = (row * width + col) * channels;
            data[at..at + channels].copy_from_slice(&noise(channels, (row * 31 + col) as u32 | 1));
            data[at] |= 1;
        }
    }
    data
}

#[test]
fn rgba_round_trips_bit_for_bit_with_transparent_margins() {
    let image = framed(37, 29, 5, 4);
    let bytes = encode_rgba(37, 29, &image).unwrap();
    assert!(is_lz4_sprite(&bytes));
    let (width, height, back) = decode_rgba(&bytes).unwrap();
    assert_eq!((width, height), (37, 29));
    assert_eq!(back, image);
}

#[test]
fn colour_hidden_under_zero_alpha_is_kept() {
    let mut image = vec![0u8; 4 * 4 * 4];
    image[(2 * 4 + 1) * 4..][..4].copy_from_slice(&[200, 100, 50, 0]);
    assert_eq!(
        decode_rgba(&encode_rgba(4, 4, &image).unwrap()).unwrap().2,
        image
    );
}

#[test]
fn indexed_round_trips_with_its_palette() {
    let indices = framed(40, 31, 3, 1);
    let palette = noise(1024, 7);
    let sprite = decode(&encode_indexed(40, 31, &indices, &palette).unwrap()).unwrap();
    let Pixels::Indexed {
        indices: back,
        palette: back_palette,
    } = sprite.pixels
    else {
        panic!("not indexed")
    };
    assert_eq!((back, back_palette), (indices, palette));
}

#[test]
fn indexed_expands_through_the_palette() {
    let palette: Vec<u8> = (0..256u32)
        .flat_map(|i| [i as u8, 255 - i as u8, 7, if i == 0 { 0 } else { 255 }])
        .collect();
    let (_, _, rgba) = decode_rgba(&encode_indexed(2, 1, &[0, 9], &palette).unwrap()).unwrap();
    assert_eq!(rgba, [0, 255, 7, 0, 9, 246, 7, 255]);
}

#[test]
fn edge_sizes_round_trip() {
    for (width, height) in [(1usize, 1usize), (1, 9), (9, 1), (3, 3)] {
        for fill in [0u8, 1] {
            let image: Vec<u8> = (0..width * height * 4)
                .map(|i| if i % 4 == 3 { fill } else { fill * 3 })
                .collect();
            assert_eq!(
                decode_rgba(&encode_rgba(width as u32, height as u32, &image).unwrap())
                    .unwrap()
                    .2,
                image,
                "{width}x{height} fill {fill}"
            );
        }
    }
    let empty = encode_indexed(0, 0, &[], &[]).unwrap();
    assert_eq!(decode(&empty).unwrap().width, 0);
}

#[test]
fn fully_transparent_sprite_stores_no_block() {
    let bytes = encode_rgba(64, 64, &vec![0u8; 64 * 64 * 4]).unwrap();
    assert_eq!(bytes.len(), 19);
    assert!(decode_rgba(&bytes).unwrap().2.iter().all(|&b| b == 0));
}

#[test]
fn hostile_input_is_an_error_not_a_panic() {
    let good = encode_rgba(16, 16, &framed(16, 16, 2, 4)).unwrap();
    assert!(decode(&good[..10]).is_err());
    assert!(decode(&good[..good.len() - 3]).is_err());
    assert!(decode(b"PNG1xxxxxxxxxxxxxxxxxxxxxxxx").is_err());
    let mut oversize = good.clone();
    oversize[7..9].copy_from_slice(&(MAX_SIDE as u16 + 1).to_le_bytes());
    assert!(decode(&oversize).is_err());
    let mut crop = good.clone();
    crop[15..17].copy_from_slice(&40u16.to_le_bytes());
    assert!(decode(&crop).is_err());
    let mut kind = good.clone();
    kind[4] = 9;
    assert!(decode(&kind).is_err());
    assert!(encode_rgba(2, 2, &[0; 7]).is_err());
    assert!(encode_indexed(2, 2, &[0; 4], &[0; 1028]).is_err());
}

#[test]
fn every_truncation_of_a_valid_sprite_fails_cleanly() {
    let good = encode_indexed(24, 24, &framed(24, 24, 4, 1), &noise(1024, 3)).unwrap();
    for cut in 0..good.len() {
        let _ = decode(&good[..cut]);
    }
}
