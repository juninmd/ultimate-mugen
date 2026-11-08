fn trim_js_ws(s: &str) -> &str {
    s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// JavaScript `parseFloat`: parses the longest numeric prefix (after leading
/// whitespace). Returns `None` where JS would return `NaN`.
pub fn parse_float(s: &str) -> Option<f64> {
    let s = trim_js_ws(s);
    let b = s.as_bytes();
    let mut i = 0;
    if matches!(b.first(), Some(b'+') | Some(b'-')) {
        i = 1;
    }
    if s[i..].starts_with("Infinity") {
        return Some(if b[0] == b'-' {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut nd = i - ds;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        nd += i - fs;
    }
    if nd == 0 {
        return None;
    }
    if i < b.len() && (b[i] | 0x20) == b'e' {
        let mut k = i + 1;
        if matches!(b.get(k), Some(b'+') | Some(b'-')) {
            k += 1;
        }
        let es = k;
        while k < b.len() && b[k].is_ascii_digit() {
            k += 1;
        }
        if k > es {
            i = k;
        }
    }
    s[..i].parse::<f64>().ok()
}

/// JavaScript `parseInt(s)` (radix 10, or 16 with a `0x` prefix). Returns
/// `None` where JS would return `NaN`. Saturates on overflow.
pub fn parse_int(s: &str) -> Option<i64> {
    let s = trim_js_ws(s);
    let b = s.as_bytes();
    let mut i = 0;
    let mut neg = false;
    if let Some(&c) = b.first() {
        if c == b'+' || c == b'-' {
            neg = c == b'-';
            i = 1;
        }
    }
    let mut radix = 10u32;
    if b.len() >= i + 2 && b[i] == b'0' && (b[i + 1] | 0x20) == b'x' {
        radix = 16;
        i += 2;
    }
    let mut v: i64 = 0;
    let mut nd = 0;
    while i < b.len() {
        let Some(d) = (b[i] as char).to_digit(radix) else {
            break;
        };
        v = v.saturating_mul(radix as i64).saturating_add(d as i64);
        nd += 1;
        i += 1;
    }
    if nd == 0 {
        return None;
    }
    Some(if neg { -v } else { v })
}

/// `parse_int` clamped to `i32`.
pub(crate) fn parse_i32(s: &str) -> Option<i32> {
    parse_int(s).map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
}
