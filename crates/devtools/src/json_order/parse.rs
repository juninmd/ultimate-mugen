use super::value::Json;

struct Parser<'a> {
    bytes: &'a [u8],
    text: &'a str,
    position: usize,
}

pub fn parse(text: &str) -> Result<Json, String> {
    let mut parser = Parser { bytes: text.as_bytes(), text, position: 0 };
    let value = parser.value()?;
    parser.skip_space();
    if parser.position != parser.bytes.len() {
        return Err(format!("trailing characters at byte {}", parser.position));
    }
    Ok(value)
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        while self.position < self.bytes.len() && self.bytes[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_space();
        if self.bytes.get(self.position) == Some(&byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", byte as char, self.position))
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.text[self.position..].starts_with(word) {
            self.position += word.len();
            Ok(value)
        } else {
            Err(format!("bad literal at byte {}", self.position))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_space();
        match self.bytes.get(self.position) {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end of input".into()),
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.position;
        while self.position < self.bytes.len() && matches!(self.bytes[self.position], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
            self.position += 1;
        }
        let raw = &self.text[start..self.position];
        raw.parse::<f64>().map_err(|_| format!("bad number at byte {start}"))?;
        Ok(Json::Num(raw.to_string()))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let rest = &self.text[self.position..];
            let c = rest.chars().next().ok_or("unterminated string")?;
            self.position += c.len_utf8();
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let escape = self.text[self.position..].chars().next().ok_or("bad escape")?;
                    self.position += escape.len_utf8();
                    match escape {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'u' => out.push(self.unicode()?),
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self.text.get(self.position..self.position + 4).ok_or("bad \\u escape")?;
        self.position += 4;
        u32::from_str_radix(digits, 16).map_err(|_| "bad \\u escape".to_string())
    }

    fn unicode(&mut self) -> Result<char, String> {
        let high = self.hex4()?;
        if (0xD800..0xDC00).contains(&high) && self.text[self.position..].starts_with("\\u") {
            self.position += 2;
            let low = self.hex4()?;
            return char::from_u32(0x10000 + ((high - 0xD800) << 10) + (low.wrapping_sub(0xDC00))).ok_or_else(|| "bad surrogate pair".to_string());
        }
        char::from_u32(high).ok_or_else(|| "bad \\u escape".to_string())
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_space();
        if self.bytes.get(self.position) == Some(&b']') {
            self.position += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_space();
            match self.bytes.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at byte {}", self.position)),
            }
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut items = Vec::new();
        self.skip_space();
        if self.bytes.get(self.position) == Some(&b'}') {
            self.position += 1;
            return Ok(Json::Obj(items));
        }
        loop {
            self.skip_space();
            let key = self.string()?;
            self.expect(b':')?;
            items.push((key, self.value()?));
            self.skip_space();
            match self.bytes.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Obj(items));
                }
                _ => return Err(format!("expected ',' or '}}' at byte {}", self.position)),
            }
        }
    }
}
