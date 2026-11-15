use crate::gfx::*;

const LAYERS: f32 = 14.0;
const LAYER_GAIN: f32 = 0.1;
const EDGE_DIMMING: f32 = 0.4;
const SEGMENTS: u32 = 48;
const RING_FRACTIONS: [f32; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];

fn layer_alpha(peak: f32, fraction: f32) -> f32 {
    peak * LAYER_GAIN * (1.0 - EDGE_DIMMING * fraction)
}

fn stacked_alpha(peak: f32, fraction: f32) -> f32 {
    let transmittance = |t: f32| {
        let u = 1.0 - layer_alpha(peak, t);
        (u * u.ln() - u) / (EDGE_DIMMING * LAYER_GAIN * peak)
    };
    1.0 - (LAYERS * (transmittance(1.0) - transmittance(fraction))).exp()
}

fn summed_alpha(peak: f32, fraction: f32) -> f32 {
    let span = 1.0 - fraction;
    LAYERS * LAYER_GAIN * peak * (span - 0.5 * EDGE_DIMMING * (1.0 - fraction * fraction))
}

pub fn falloff(peak: f32, fraction: f32, blend: Blend) -> f32 {
    if peak <= 0.0 {
        return 0.0;
    }
    let peak = peak.min(1.0 / LAYER_GAIN - 1e-3);
    match blend {
        Blend::Alpha => stacked_alpha(peak, fraction),
        Blend::Add | Blend::Sub => summed_alpha(peak, fraction),
    }
    .clamp(0.0, 1.0)
}

pub fn glow(x: f32, y: f32, r: f32, c: Color) {
    if c.a <= 0.0 || r <= 0.0 {
        return;
    }
    let blend = current_blend();
    let stops = RING_FRACTIONS.map(|fraction| {
        (
            fraction,
            Color::new(c.r, c.g, c.b, falloff(c.a, fraction, blend)),
        )
    });
    draw_radial_gradient(x, y, r, SEGMENTS, &stops);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disc_stack(peak: f32, fraction: f32) -> f32 {
        let mut transmittance = 1.0;
        for i in 0..LAYERS as usize {
            let t = 1.0 - i as f32 / LAYERS;
            if t >= fraction {
                transmittance *= 1.0 - layer_alpha(peak, t);
            }
        }
        1.0 - transmittance
    }

    #[test]
    fn the_gradient_matches_the_fourteen_disc_stack_it_replaced() {
        for peak in [0.1, 0.35, 0.8, 1.0] {
            for step in 0..=10 {
                let fraction = step as f32 / 10.0;
                let (smooth, stepped) = (
                    falloff(peak, fraction, Blend::Alpha),
                    disc_stack(peak, fraction),
                );
                assert!(
                    (smooth - stepped).abs() <= layer_alpha(peak, 0.0),
                    "peak {peak} at {fraction}: {smooth} vs {stepped}"
                );
            }
        }
    }

    #[test]
    fn it_fades_to_nothing_at_the_rim_and_is_brightest_at_the_core() {
        for blend in [Blend::Alpha, Blend::Add] {
            assert!(falloff(0.5, 1.0, blend).abs() < 1e-4);
            let samples: Vec<f32> = RING_FRACTIONS
                .iter()
                .map(|f| falloff(0.5, *f, blend))
                .collect();
            assert!(samples.windows(2).all(|w| w[0] > w[1]));
        }
        assert_eq!(falloff(0.0, 0.0, Blend::Alpha), 0.0);
    }
}
