//! Port of the synthetic tests in tests/decoders.test.ts.

mod common;
use common::*;
use ume_formats::act::{decode_act, encode_act};
use ume_formats::pcx::{decode_pcx, decode_pcx_rle};
use ume_formats::sff::{decode_lz5, decode_png, decode_rle5, decode_rle8, decode_sff, sprite_to_rgba};
use ume_formats::snd::decode_snd;
use std::sync::Arc;

// ---------------------------------------------------------------- PCX

#[test]
fn pcx_round_trip_odd_width_and_palette() {
    let (w, h) = (37usize, 11usize);
    let mut r = rng(1);
    let px: Vec<u8> = (0..w * h)
        .map(|i| {
            if i % 7 < 3 {
                0xc5
            } else if r() > 128 {
                r()
            } else {
                0
            }
        })
        .collect();
    let pal = make_rgb_palette(2);
    let img = decode_pcx(&encode_pcx(w, h, &px, Some(&pal)), true).unwrap();
    assert_eq!(img.width, w as u32);
    assert_eq!(img.height, h as u32);
    assert_eq!(img.bytes_per_line, 38);
    assert_eq!(img.pixels, px);
    let p = img.palette.expect("palette");
    assert_eq!(p[5 * 4], pal[15]);
    assert_eq!(p[255 * 4 + 2], pal[767]);
    assert_eq!(p[3], 0);
    assert_eq!(p[7], 255);
}

#[test]
fn pcx_cuts_runs_at_scanline_boundaries() {
    let data = [0xc4, 9, 0xc2, 7];
    let (px, end) = decode_pcx_rle(&data, 2, 2, 2, 0, data.len());
    assert_eq!(px, vec![9, 9, 7, 7]);
    assert_eq!(end, 4);
}

// ---------------------------------------------------------------- RLE8 / RLE5 / LZ5

#[test]
fn rle8_hand_made() {
    let out = decode_rle8(&[0x43, 5, 7, 0x41, 0x42, 0x80], 6);
    assert_eq!(out, vec![5, 5, 5, 7, 0x42, 0x80]);
}

#[test]
fn rle8_round_trip() {
    let mut r = rng(3);
    let px: Vec<u8> = (0..500).map(|i| if i % 13 < 6 { 0x44 } else { r() }).collect();
    assert_eq!(decode_rle8(&encode_rle8(&px), px.len()), px);
}

#[test]
fn rle5_hand_made() {
    let stream = [
        2,
        0x80 | 2,
        7,
        (1 << 5) | 3,
        5, // 7 x3, then 3 x2, 5 x1
        1,
        0x00, // color 0 x2
        0,
        0x80 | 1,
        30,
        (7 << 5) | 31, // 30 x1, 31 x8
    ];
    let out = decode_rle5(&stream, 17);
    assert_eq!(out, vec![7, 7, 7, 3, 3, 5, 0, 0, 30, 31, 31, 31, 31, 31, 31, 31, 31]);
}

#[test]
fn lz5_all_packet_kinds() {
    let stream = [
        0xf8, // control: packets 0-2 RLE, 3-7 LZ
        0x41, // RLE short: 1 x2
        0x22, // RLE short: 2 x1
        0x03, 0x00, // RLE long: 3 x8
        0x02, 0x0a, // LZ short: len 3, offset 11
        0x01, 0x0d, // LZ short: len 2, offset 14
        0xc1, 0x00, // LZ short: len 2, offset 1 (overlapping)
        0xc1, // 4th LZ short: recycled-bit offset 16
        0x00, 0x13, 0x00, // LZ long: offset 20, len 3
        0x00, // control: all RLE
        0x3f, // RLE short: 31 x1
    ];
    let out = decode_lz5(&stream, 24);
    assert_eq!(out, vec![1, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 2, 1, 1, 1, 1, 2, 3, 1, 1, 2, 31]);
}

#[test]
fn lz5_long_packet_offset_high_bits() {
    let stream = [0x04, 0x27, 0x01, 0xff, 0x40, 0x07, 0x00];
    let out = decode_lz5(&stream, 267);
    assert_eq!(out[0], 7);
    assert_eq!(out[263], 1);
    assert_eq!(&out[264..], &[7, 1, 1]);
}

// ---------------------------------------------------------------- PNG

const PW: usize = 13;
const PH: usize = 10;

#[test]
fn png_rgba_all_filters() {
    let mut r = rng(4);
    let data: Vec<u8> = (0..PW * PH * 4).map(|_| r()).collect();
    let png = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 6, bit_depth: 8, data: &data, plte: None, trns: None })).unwrap();
    assert_eq!(png.width, PW as u32);
    assert_eq!(png.rgba.unwrap(), data);
}

#[test]
fn png_rgb() {
    let mut r = rng(5);
    let data: Vec<u8> = (0..PW * PH * 3).map(|_| r()).collect();
    let png = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 2, bit_depth: 8, data: &data, plte: None, trns: None })).unwrap();
    let rgba = png.rgba.unwrap();
    for i in 0..PW * PH {
        assert_eq!(&rgba[i * 4..i * 4 + 4], &[data[i * 3], data[i * 3 + 1], data[i * 3 + 2], 255]);
    }
}

#[test]
fn png_gray_and_gray_alpha() {
    let mut r = rng(9);
    let g: Vec<u8> = (0..PW * PH).map(|_| r()).collect();
    let png = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 0, bit_depth: 8, data: &g, plte: None, trns: None })).unwrap();
    let rgba = png.rgba.unwrap();
    for i in 0..PW * PH {
        assert_eq!(&rgba[i * 4..i * 4 + 4], &[g[i], g[i], g[i], 255]);
    }
    let ga: Vec<u8> = (0..PW * PH * 2).map(|_| r()).collect();
    let png = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 4, bit_depth: 8, data: &ga, plte: None, trns: None })).unwrap();
    let rgba = png.rgba.unwrap();
    for i in 0..PW * PH {
        assert_eq!(&rgba[i * 4..i * 4 + 4], &[ga[i * 2], ga[i * 2], ga[i * 2], ga[i * 2 + 1]]);
    }
}

#[test]
fn png_indexed_8bit_and_4bit() {
    let mut r = rng(6);
    let idx: Vec<u8> = (0..PW * PH).map(|_| r() & 15).collect();
    let pal = make_rgb_palette(7);
    let plte = &pal[..16 * 3];
    let trns = [0u8, 128];
    let png = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 3, bit_depth: 8, data: &idx, plte: Some(plte), trns: Some(&trns) })).unwrap();
    assert_eq!(png.indexed.as_deref().unwrap(), &idx);
    let p = png.palette.unwrap();
    assert_eq!(p[3], 0);
    assert_eq!(p[7], 128);
    assert_eq!(p[11], 255);
    assert_eq!(p[4 * 4 + 1], plte[4 * 3 + 1]);
    // 4-bit packing
    let stride = PW.div_ceil(2);
    let mut packed = vec![0u8; stride * PH];
    for y in 0..PH {
        for x in 0..PW {
            packed[y * stride + (x >> 1)] |= idx[y * PW + x] << (if x & 1 == 1 { 0 } else { 4 });
        }
    }
    let png4 = decode_png(&encode_png(PngOpts { w: PW, h: PH, color_type: 3, bit_depth: 4, data: &packed, plte: Some(plte), trns: None })).unwrap();
    assert_eq!(png4.indexed.unwrap(), idx);
}

#[test]
fn png_rejects_garbage() {
    assert!(decode_png(b"not a png").is_err());
}

// ---------------------------------------------------------------- SFF v1

struct V1Spr {
    group: u32,
    index: u32,
    ax: i32,
    ay: i32,
    img: Option<(usize, usize, Vec<u8>)>,
    pal: Option<Vec<u8>>,
    same_pal: bool,
    link: Option<u32>,
}

fn encode_sff_v1(sprites: &[V1Spr]) -> Vec<u8> {
    let mut wr = Writer::new();
    wr.str("ElecbyteSpr\0").u8(0).u8(1).u8(0).u8(1);
    wr.u32(1).u32(sprites.len() as u32).u32(512).u32(32).u8(1).pad(512);
    for (i, s) in sprites.iter().enumerate() {
        let pcx = match (&s.img, s.link) {
            (Some((w, h, px)), None) => encode_pcx(*w, *h, px, s.pal.as_deref()),
            _ => Vec::new(),
        };
        let start = wr.len();
        let next = if i == sprites.len() - 1 { 0 } else { start + 32 + pcx.len() };
        wr.u32(next as u32).u32(pcx.len() as u32).u16(s.ax as u32).u16(s.ay as u32).u16(s.group).u16(s.index);
        wr.u16(s.link.unwrap_or(0)).u8(s.same_pal as u32);
        wr.pad(start + 32).raw(&pcx);
    }
    wr.out()
}

#[test]
fn sff_v1_round_trip() {
    let mut r = rng(8);
    let mut mk = |w: usize, h: usize| -> Vec<u8> { (0..w * h).map(|i| if i % 5 == 0 { 0 } else { r() }).collect() };
    let (pal_a, pal_b) = (make_rgb_palette(10), make_rgb_palette(11));
    let (px0, px1, px3) = (mk(21, 30), mk(8, 8), mk(16, 4));
    let spr = |group, index, ax, ay| V1Spr { group, index, ax, ay, img: None, pal: None, same_pal: false, link: None };
    let buf = encode_sff_v1(&[
        V1Spr { img: Some((21, 30, px0.clone())), pal: Some(pal_a.clone()), ..spr(0, 0, 10, 29) },
        V1Spr { img: Some((8, 8, px1.clone())), same_pal: true, ..spr(0, 1, -3, 4) },
        V1Spr { link: Some(0), ..spr(5, 2, 7, 8) },
        V1Spr { img: Some((16, 4, px3.clone())), pal: Some(pal_b.clone()), ..spr(9000, 0, 0, 0) },
    ]);
    let sff = decode_sff(&buf).unwrap();
    assert_eq!(sff.version, 1);
    assert_eq!(sff.sprites.len(), 4);
    let (s0, s1, s2, s3) = (&sff.sprites[0], &sff.sprites[1], &sff.sprites[2], &sff.sprites[3]);
    assert_eq!((s0.group, s0.index, s0.axis_x, s0.axis_y, s0.width, s0.height), (0, 0, 10, 29, 21, 30));
    assert_eq!(s0.indexed.as_deref().unwrap(), &px0);
    assert_eq!(s0.palette.as_ref().unwrap()[4], pal_a[3]);
    assert_eq!((s1.axis_x, s1.axis_y, s1.width, s1.height), (-3, 4, 8, 8));
    assert_eq!(s1.indexed.as_deref().unwrap(), &px1);
    assert!(Arc::ptr_eq(s1.palette.as_ref().unwrap(), s0.palette.as_ref().unwrap()));
    assert_eq!((s2.group, s2.index, s2.axis_x, s2.axis_y, s2.width, s2.height), (5, 2, 7, 8, 21, 30));
    assert_eq!(s2.indexed, s0.indexed);
    assert_eq!(s3.indexed.as_deref().unwrap(), &px3);
    assert_eq!(s3.palette.as_ref().unwrap()[255 * 4 + 1], pal_b[766]);
    assert_eq!(sff.palettes.len(), 2);
    assert_eq!(s3.palette_index, 1);
    let rgba = sprite_to_rgba(s0, None);
    let c = s0.indexed.as_ref().unwrap()[1] as usize;
    let expect = if c != 0 { [pal_a[c * 3], pal_a[c * 3 + 1], pal_a[c * 3 + 2], 255] } else { [0, 0, 0, 0] };
    assert_eq!(&rgba[4..8], &expect);
}

// ---------------------------------------------------------------- SFF v2

#[test]
fn sff_v2_synthetic() {
    let (w, h) = (6usize, 4usize);
    let mut r = rng(12);
    let px_raw: Vec<u8> = (0..w * h).map(|_| r()).collect();
    let px_rle8: Vec<u8> = (0..w * h).map(|i| if i < 10 { 3 } else { r() }).collect();
    let rle5 = vec![23u8, 0x80, 9]; // 24 x color 9
    let lz5 = vec![0x00u8, 0x03, 16]; // RLE long: 3 x 24
    let png8 = encode_png(PngOpts { w, h, color_type: 3, bit_depth: 8, data: &px_raw, plte: Some(&make_rgb_palette(1)), trns: None });
    let rgba_data: Vec<u8> = (0..w * h * 4).map(|_| r()).collect();
    let png32 = encode_png(PngOpts { w, h, color_type: 6, bit_depth: 8, data: &rgba_data, plte: None, trns: None });
    let with_len = |d: &[u8]| -> Vec<u8> {
        let mut v = vec![(w * h) as u8, 0, 0, 0];
        v.extend_from_slice(d);
        v
    };
    let blobs = [px_raw.clone(), with_len(&encode_rle8(&px_rle8)), with_len(&rle5), with_len(&lz5), with_len(&png8), with_len(&png32)];
    let formats = [0u32, 2, 3, 4, 10, 12];
    let depths = [8u32, 8, 5, 5, 8, 32];
    let pal0: Vec<u8> = (0..1024).map(|i| ((i * 7) & 0xff) as u8).collect();

    let mut wr = Writer::new();
    wr.str("ElecbyteSpr\0").u8(0).u8(1).u8(0).u8(2).pad(36);
    let n_spr = blobs.len() + 1;
    let spr_ofs = 72;
    let pal_ofs = spr_ofs + n_spr * 28;
    let ldata_ofs = pal_ofs + 2 * 16;
    wr.u32(spr_ofs as u32).u32(n_spr as u32).u32(pal_ofs as u32).u32(2).u32(ldata_ofs as u32).u32(0).u32(0).u32(0).pad(spr_ofs);
    let mut data_ofs = pal0.len();
    for (i, b) in blobs.iter().enumerate() {
        wr.u16(1).u16(i as u32).u16(w as u32).u16(h as u32).u16(i as u32).u16(0xffff - 1).u16(0).u8(formats[i]).u8(depths[i]);
        wr.u32(data_ofs as u32).u32(b.len() as u32).u16(if i == 4 { 1 } else { 0 }).u16(0);
        data_ofs += b.len();
    }
    wr.u16(2).u16(0).u16(0).u16(0).u16(0).u16(0).u16(0).u8(0).u8(0).u32(0).u32(0).u16(0).u16(0); // linked to sprite 0
    wr.u16(1).u16(1).u16(256).u16(0).u32(0).u32(1024); // palette 0
    wr.u16(1).u16(2).u16(256).u16(0).u32(0).u32(0); // palette 1 linked to 0
    wr.raw(&pal0);
    for b in &blobs {
        wr.raw(b);
    }

    let sff = decode_sff(&wr.out()).unwrap();
    assert_eq!(sff.version, 2);
    assert_eq!(sff.palettes.len(), 2);
    assert!(Arc::ptr_eq(&sff.palettes[1].data, &sff.palettes[0].data));
    assert_eq!(sff.palettes[0].data[3], 0);
    assert_eq!(sff.palettes[0].data[4 + 3], pal0[7]); // v2.01 keeps stored alpha
    let s = &sff.sprites;
    assert_eq!(s.len(), 7);
    assert_eq!((s[1].axis_x, s[1].axis_y), (1, -2));
    assert_eq!(s[0].indexed.as_deref().unwrap(), &px_raw);
    assert_eq!(s[1].indexed.as_deref().unwrap(), &px_rle8);
    assert_eq!(s[2].indexed.as_deref().unwrap(), &vec![9u8; 24]);
    assert_eq!(s[3].indexed.as_deref().unwrap(), &vec![3u8; 24]);
    assert_eq!(s[4].indexed.as_deref().unwrap(), &px_raw);
    assert!(Arc::ptr_eq(s[4].palette.as_ref().unwrap(), &sff.palettes[1].data)); // SFF palette wins over PLTE
    assert_eq!(s[5].rgba.as_deref().unwrap(), &rgba_data);
    assert_eq!(s[5].palette_index, -1);
    assert_eq!(s[6].indexed, s[0].indexed);
    assert_eq!((s[6].group, s[6].width, s[6].height), (2, w as u32, h as u32));
}

#[test]
fn sff_rejects_bad_signature() {
    assert!(decode_sff(b"NotAnSffFile....").is_err());
    assert!(decode_sff(&[]).is_err());
}

// ---------------------------------------------------------------- SND / ACT

#[test]
fn snd_walks_linked_list() {
    let mut wr = Writer::new();
    wr.str("ElecbyteSnd\0").u16(0).u16(1).u32(2).u32(512).pad(512);
    let a = [82u8, 73, 70, 70, 1, 2, 3];
    let b = [82u8, 73, 70, 70, 9];
    wr.u32((512 + 16 + a.len()) as u32).u32(a.len() as u32).u32(5).u32(1).raw(&a);
    wr.u32(0).u32(b.len() as u32).u32(-1i32 as u32).u32(42).raw(&b);
    let m = decode_snd(&wr.out());
    assert_eq!(m.len(), 2);
    assert_eq!(m[&(5, 1)], a);
    assert_eq!(m[&(-1, 42)], b);
    assert!(decode_snd(b"garbage").is_empty());
}

#[test]
fn act_reverse_order() {
    let mut raw = vec![0u8; 768];
    raw[0..3].copy_from_slice(&[10, 20, 30]);
    raw[255 * 3..256 * 3].copy_from_slice(&[1, 2, 3]);
    raw[254 * 3..255 * 3].copy_from_slice(&[4, 5, 6]);
    let pal = decode_act(&raw);
    assert_eq!(pal.len(), 1024);
    assert_eq!(&pal[255 * 4..256 * 4], &[10, 20, 30, 255]);
    assert_eq!(&pal[0..4], &[1, 2, 3, 0]);
    assert_eq!(&pal[4..8], &[4, 5, 6, 255]);
    assert_eq!(encode_act(&pal), raw);
}
