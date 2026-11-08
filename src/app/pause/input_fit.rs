pub fn input_width(input: &str, size: f32, button_w: &dyn Fn(char, f32) -> f32) -> f32 {
    let mut w = 0.0;
    for tok in input
        .replace(['~', ' ', '/', '$', '>'], "")
        .split(',')
        .filter(|t| !t.is_empty())
    {
        for (n, part) in tok.split('+').enumerate() {
            if n > 0 {
                w += size * 0.8 + 8.0;
            }
            let is_dir = matches!(part, "F" | "B" | "U" | "D" | "DF" | "DB" | "UF" | "UB");
            w += match (is_dir, part.chars().next()) {
                (true, _) => size + 4.0,
                (false, Some(c)) => button_w(c, size) + 4.0,
                _ => 0.0,
            };
        }
    }
    w
}

pub fn fit_input_size(
    input: &str,
    max_w: f32,
    base: f32,
    min: f32,
    button_w: &dyn Fn(char, f32) -> f32,
) -> f32 {
    let mut s = base;
    while s > min && input_width(input, s, button_w) > max_w {
        s -= 1.0;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_inputs_shrink_to_fit_the_column() {
        let long = "~D, DF, F, D, DF, F, x+y";
        let wide = |_: char, s: f32| s * 1.5;
        assert!(input_width(long, 28.0, &wide) > 250.0);
        let s = fit_input_size(long, 200.0, 28.0, 14.0, &wide);
        assert!(s < 28.0 && input_width(long, s, &wide) <= 200.0);
        assert_eq!(fit_input_size("x", 200.0, 28.0, 14.0, &wide), 28.0);
    }
}
