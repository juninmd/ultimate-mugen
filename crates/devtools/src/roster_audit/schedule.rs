use super::{Job, Options};

const FOE_OFFSETS: [usize; 3] = [1, 7, 37];
const STAGE_OFFSETS: [usize; 3] = [0, 1, 2];

#[derive(Default)]
pub struct Plan {
    chars: Vec<(String, String)>,
    stages: Vec<(String, String)>,
    stage_fights: usize,
    forced: Option<(String, String)>,
}

impl Plan {
    pub fn new(char_jobs: &[Job], stage_jobs: &[Job], opts: &Options) -> Plan {
        let forced = opts.opponent.as_ref().and_then(|needle| {
            let needle = needle.to_lowercase();
            char_jobs.iter().find(|j| j.def.to_lowercase().contains(&needle) || j.dir.to_lowercase().contains(&needle)).map(|j| (j.def.clone(), j.dir.clone()))
        });
        Plan {
            chars: char_jobs.iter().map(|j| (j.def.clone(), j.dir.clone())).collect(),
            stages: stage_jobs.iter().map(|j| (j.def.clone(), j.dir.clone())).collect(),
            stage_fights: opts.stage_fights.max(1),
            forced,
        }
    }

    fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    fn index_of(&self, def: &str) -> usize {
        self.chars.iter().position(|(d, _)| d == def).unwrap_or(0)
    }

    fn pick(&self, index: usize, offset: usize) -> (String, String) {
        let n = self.chars.len().max(1);
        let at = (index + offset) % n;
        self.chars.get(at).cloned().unwrap_or_default()
    }

    pub fn foes(&self, def: &str) -> Vec<(String, String, String, String)> {
        if let Some((foe_def, foe_dir)) = &self.forced {
            let stage = self.stage_for(self.index_of(def));
            return vec![(foe_def.clone(), foe_dir.clone(), stage.0, stage.1)];
        }
        let index = self.index_of(def);
        FOE_OFFSETS
            .iter()
            .enumerate()
            .map(|(slot, offset)| {
                let (foe_def, foe_dir) = self.pick(index, *offset);
                let stage = self.stage_for(index + slot);
                (foe_def, foe_dir, stage.0, stage.1)
            })
            .collect()
    }

    pub fn stage_pair(&self, def: &str) -> (String, String, String, String) {
        let stage_index = self.stages.iter().position(|(d, _)| d == def).unwrap_or(0);
        let left = (stage_index * 3 + 1) % self.chars.len().max(1);
        let (a_def, a_dir) = self.pick(left, 0);
        let (b_def, b_dir) = self.pick(left, self.stage_fights);
        (a_def, a_dir, b_def, b_dir)
    }

    fn stage_for(&self, slot: usize) -> (String, String) {
        if self.stages.is_empty() {
            return (String::new(), String::new());
        }
        let at = (slot * STAGE_OFFSETS[slot % STAGE_OFFSETS.len()]) % self.stages.len();
        self.stages[at].clone()
    }
}
