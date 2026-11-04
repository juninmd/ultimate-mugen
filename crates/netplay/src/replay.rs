const MAGIC: &[u8; 4] = b"UMRP";
const VERSION: u32 = 1;

const MAX_ENTRIES: usize = 1 << 24;
const LIST_SEPARATOR: char = '\u{1f}';

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub build: String,
    pub mode: String,
    pub stage: String,
    pub teams: [Vec<String>; 2],
    pub palettes: Vec<i32>,
    pub simul: bool,
    pub seed: u32,
    pub created: u64,
    pub checksum: Option<u128>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Replay {
    pub players: usize,
    pub meta: Vec<(String, String)>,
    pub frames: Vec<u32>,
}

impl Replay {
    pub fn new(players: usize) -> Self {
        Replay { players: players.max(1), ..Default::default() }
    }

    pub fn push(&mut self, inputs: &[u32]) {
        self.frames.extend((0..self.players).map(|i| inputs.get(i).copied().unwrap_or(0)));
    }

    pub fn len(&self) -> usize {
        self.frames.len() / self.players.max(1)
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn frame(&self, n: usize) -> Option<&[u32]> {
        self.frames.get(n * self.players..(n + 1) * self.players)
    }

    pub fn meta(&self, key: &str) -> Option<&str> {
        self.meta.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn with_header(players: usize, header: &Header) -> Self {
        let mut replay = Replay::new(players);
        replay.set_header(header);
        replay
    }

    pub fn set_meta(&mut self, key: &str, value: String) {
        match self.meta.iter_mut().find(|(k, _)| k == key) {
            Some(entry) => entry.1 = value,
            None => self.meta.push((key.to_string(), value)),
        }
    }

    pub fn set_header(&mut self, h: &Header) {
        let join = |items: &[String]| items.join(&LIST_SEPARATOR.to_string());
        self.set_meta("build", h.build.clone());
        self.set_meta("mode", h.mode.clone());
        self.set_meta("stage", h.stage.clone());
        self.set_meta("team1", join(&h.teams[0]));
        self.set_meta("team2", join(&h.teams[1]));
        self.set_meta("palettes", h.palettes.iter().map(i32::to_string).collect::<Vec<_>>().join(","));
        self.set_meta("simul", (h.simul as u8).to_string());
        self.set_meta("seed", h.seed.to_string());
        self.set_meta("created", h.created.to_string());
        if let Some(sum) = h.checksum {
            self.set_meta("checksum", format!("{sum:032x}"));
        }
    }

    pub fn header(&self) -> Result<Header, String> {
        let text = |key: &str| self.meta(key).map(str::to_string).ok_or_else(|| format!("replay: missing {key}"));
        let list = |key: &str| -> Result<Vec<String>, String> {
            let raw = text(key)?;
            Ok(if raw.is_empty() { vec![] } else { raw.split(LIST_SEPARATOR).map(str::to_string).collect() })
        };
        let number = |key: &str| -> Result<u64, String> { text(key)?.parse().map_err(|_| format!("replay: bad {key}")) };
        let palettes = text("palettes")?
            .split(',')
            .filter(|p| !p.is_empty())
            .map(|p| p.parse().map_err(|_| "replay: bad palettes".to_string()))
            .collect::<Result<Vec<i32>, _>>()?;
        let checksum = match self.meta("checksum") {
            Some(raw) => Some(u128::from_str_radix(raw, 16).map_err(|_| "replay: bad checksum".to_string())?),
            None => None,
        };
        Ok(Header {
            build: text("build")?,
            mode: text("mode")?,
            stage: text("stage")?,
            teams: [list("team1")?, list("team2")?],
            palettes,
            simul: number("simul")? != 0,
            seed: number("seed")? as u32,
            created: number("created")?,
            checksum,
        })
    }

    pub fn check_build(&self, current: &str) -> Result<(), String> {
        match self.meta("build") {
            Some(build) if build == current => Ok(()),
            Some(build) => Err(format!("This replay was recorded with build {build}; this is build {current}. Replays only play on the build that recorded them.")),
            None => Err("This replay has no build stamp and cannot be played back.".into()),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.frames.len() * 4);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&(self.players as u32).to_le_bytes());
        out.extend_from_slice(&(self.meta.len() as u32).to_le_bytes());
        for (k, v) in &self.meta {
            for s in [k, v] {
                out.extend_from_slice(&(s.len() as u32).to_le_bytes());
                out.extend_from_slice(s.as_bytes());
            }
        }
        out.extend_from_slice(&(self.len() as u32).to_le_bytes());
        let mut prev = vec![0u32; self.players];
        let mut run = 0u32;
        let flush = |out: &mut Vec<u8>, run: &mut u32, prev: &[u32]| {
            if *run > 0 {
                out.extend_from_slice(&run.to_le_bytes());
                for v in prev {
                    out.extend_from_slice(&v.to_le_bytes());
                }
                *run = 0;
            }
        };
        for f in self.frames.chunks(self.players) {
            if run > 0 && f != prev.as_slice() {
                flush(&mut out, &mut run, &prev);
            }
            prev.copy_from_slice(f);
            run += 1;
        }
        flush(&mut out, &mut run, &prev);
        out
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let mut c = Cursor { b, at: 0 };
        if c.take(4)? != MAGIC {
            return Err("replay: bad magic".into());
        }
        let version = c.u32()?;
        if version != VERSION {
            return Err(format!("replay: unsupported version {version}"));
        }
        let players = c.u32()? as usize;
        if players == 0 || players > 64 {
            return Err("replay: bad player count".into());
        }
        let n_meta = c.u32()?;
        let mut meta = Vec::new();
        for _ in 0..n_meta {
            let k = c.string()?;
            let v = c.string()?;
            meta.push((k, v));
        }
        let total = c.u32()? as usize;
        if total.saturating_mul(players) > MAX_ENTRIES {
            return Err("replay: too long".into());
        }
        let mut frames = Vec::with_capacity(total.min(1 << 16) * players);
        while frames.len() < total * players {
            let run = c.u32()? as usize;
            if run == 0 {
                return Err("replay: empty run".into());
            }
            let f = (0..players).map(|_| c.u32()).collect::<Result<Vec<_>, _>>()?;
            for _ in 0..run.min(total - frames.len() / players) {
                frames.extend_from_slice(&f);
            }
        }
        frames.truncate(total * players);
        Ok(Replay { players, meta, frames })
    }
}

struct Cursor<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let s = self.b.get(self.at..self.at.saturating_add(n)).ok_or("replay: truncated")?;
        self.at += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> Result<String, String> {
        let len = self.u32()? as usize;
        Ok(String::from_utf8_lossy(self.take(len)?).into_owned())
    }
}
