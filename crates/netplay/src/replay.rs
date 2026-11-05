const MAGIC: &[u8; 4] = b"UMRP";
const VERSION: u32 = 1;
const MAX_REPLAY_BYTES: usize = 64 * 1024 * 1024;
const MAX_REPLAY_FRAMES_BYTES: usize = 64 * 1024 * 1024;
const MAX_REPLAY_METADATA: u32 = 16_384;

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
        if self.players == 0 {
            return None;
        }
        let start = n.checked_mul(self.players)?;
        let end = start.checked_add(self.players)?;
        self.frames.get(start..end)
    }

    pub fn meta(&self, key: &str) -> Option<&str> {
        self.meta.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
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
        if b.len() > MAX_REPLAY_BYTES {
            return Err("replay: file exceeds size limit".into());
        }
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
        if n_meta > MAX_REPLAY_METADATA {
            return Err("replay: too many metadata entries".into());
        }
        let mut meta = Vec::new();
        for _ in 0..n_meta {
            let k = c.string()?;
            let v = c.string()?;
            meta.push((k, v));
        }
        let total = c.u32()? as usize;
        let frame_values = total.checked_mul(players).ok_or("replay: frame count overflow")?;
        let frame_bytes = frame_values.checked_mul(4).ok_or("replay: frame size overflow")?;
        if frame_bytes > MAX_REPLAY_FRAMES_BYTES {
            return Err("replay: decoded frames exceed size limit".into());
        }
        let mut frames = Vec::with_capacity(frame_values);
        let mut decoded_frames = 0usize;
        while decoded_frames < total {
            let run = c.u32()? as usize;
            if run == 0 || run > total - decoded_frames {
                return Err("replay: invalid frame run".into());
            }
            let f = (0..players).map(|_| c.u32()).collect::<Result<Vec<_>, _>>()?;
            for _ in 0..run {
                frames.extend_from_slice(&f);
            }
            decoded_frames += run;
        }
        if c.at != b.len() {
            return Err("replay: trailing bytes".into());
        }
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
