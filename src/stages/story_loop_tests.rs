use super::*;
use crate::stages::{decode_illustration, STORY_ARENAS};

fn decode(path: &str) -> Arc<ImageData> {
    let root = env!("CARGO_MANIFEST_DIR");
    decode_illustration(&std::fs::read(format!("{root}/assets/stages/{path}")).unwrap()).unwrap()
}

#[test]
fn runs_tile_the_whole_strip_without_gaps() {
    for core in CORES {
        let runs = runs(core);
        let mut at = 0.0;
        for run in &runs {
            assert_eq!(run.at, at);
            assert!(run.source >= 0.0 && run.source + run.width <= SEGMENT);
            at += run.width;
        }
        assert_eq!(at, STRIP);
    }
}

#[test]
fn mirror_chains_meet_on_the_same_source_column() {
    let runs = runs(CORES[0]);
    for pair in runs.windows(2).filter(|pair| pair[1].at % SEGMENT != 0.0) {
        let edge = pair[1].at;
        let gap = sample(&runs, edge).unwrap() - sample(&runs, edge - 1.0).unwrap();
        assert!(gap.abs() <= 1.0);
    }
}

#[test]
fn pieces_cover_a_range_in_both_directions() {
    let runs = runs(CORES[1]);
    for mirrored in [false, true] {
        let parts = pieces(&runs, 1600.0, 1900.0, mirrored);
        assert_eq!(parts.iter().map(|piece| piece.width).sum::<f32>(), 300.0);
        let mut offsets: Vec<f32> = parts.iter().map(|piece| piece.offset).collect();
        offsets.sort_by(f32::total_cmp);
        assert_eq!(offsets[0], 0.0);
    }
    let plain = pieces(&runs, 900.0, 905.0, false)[0];
    let flipped = pieces(&runs, 900.0, 905.0, true)[0];
    assert_eq!(plain.source, flipped.source);
    assert_ne!(plain.flipped, flipped.flipped);
}

#[test]
fn seam_mix_crosses_half_at_the_seam() {
    assert!(seam_mix(-SEAM_BAND) < 0.01);
    assert!(seam_mix(SEAM_BAND) > 0.99);
    assert!((seam_mix(-0.5) - 0.5).abs() < 0.02);
}

#[test]
fn layout_reproduces_the_static_strip_outside_seams_and_props() {
    for (index, arena) in STORY_ARENAS.iter().enumerate() {
        let base = decode(&format!("{arena}.png"));
        let strip = decode(&format!("story/{arena}.png"));
        let runs = runs(CORES[index]);
        let texel = base.w as f32 / SEGMENT;
        let mut x = 3.0;
        while x < STRIP {
            let seam = (x / SEGMENT).round() * SEGMENT;
            if (x - seam).abs() > SEAM_BAND || seam == 0.0 || seam == STRIP {
                let source = (sample(&runs, x).unwrap() * texel + 1.0) as usize;
                let grade = grade_at(GRADES[index], x);
                for y in [12usize, 40, 90] {
                    let from = (y * 2 + 1) * base.w as usize * 4 + source * 4;
                    let to = y * strip.w as usize * 4 + x as usize * 4;
                    for channel in 0..3 {
                        let want = (base.rgba[from + channel] as f32 * grade[channel]).min(255.0);
                        let got = strip.rgba[to + channel] as f32;
                        assert!((want - got).abs() <= 3.0, "{arena} x={x} y={y}");
                    }
                }
            }
            x += 37.0;
        }
    }
}
