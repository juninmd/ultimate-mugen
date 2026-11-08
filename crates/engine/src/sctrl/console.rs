use crate::chardef::Controller;
use crate::matchs::Match;
use std::collections::VecDeque;

pub const CONSOLE_MAX_LINES: usize = 200;
pub const CONSOLE_LINES_PER_SECOND: u32 = 20;
pub const CONSOLE_LINE_MAX_CHARS: usize = 240;
const WINDOW_TICKS: i64 = 60;

#[derive(Clone, Debug, Default)]
pub struct ConsoleLog {
    lines: VecDeque<String>,
    window_start: i64,
    window_count: u32,
    dropped: u32,
    warned: Vec<String>,
}

impl ConsoleLog {
    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn dropped(&self) -> u32 {
        self.dropped
    }

    pub fn print(&mut self, now: i64, line: &str) -> bool {
        if now - self.window_start >= WINDOW_TICKS || now < self.window_start {
            self.window_start = now;
            self.window_count = 0;
        }
        if self.window_count >= CONSOLE_LINES_PER_SECOND {
            self.dropped += 1;
            return false;
        }
        self.window_count += 1;
        if self.lines.len() >= CONSOLE_MAX_LINES {
            self.lines.pop_front();
        }
        self.lines
            .push_back(line.chars().take(CONSOLE_LINE_MAX_CHARS).collect());
        true
    }

    pub fn warn_once(&mut self, key: &str, message: &str) -> bool {
        if self.warned.iter().any(|k| k == key) {
            return false;
        }
        self.warned.push(key.to_string());
        if self.lines.len() >= CONSOLE_MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(format!("warning: {message}"));
        true
    }
}

pub fn print_to_console(m: &mut Match, i: usize, sc: &Controller) {
    let template = sc
        .raw("text")
        .map(ume_formats::ini::unquote)
        .unwrap_or_default();
    let args: Vec<f64> = sc
        .param("params")
        .map(|p| {
            (0..p.exprs.len())
                .map(|k| m.num(i, sc, "params", 0.0, k))
                .collect()
        })
        .unwrap_or_default();
    let line = crate::ikemen::format_text(&template, &args);
    let now = m.game_time;
    for part in line.split('\n') {
        m.console.print(now, part);
    }
}
