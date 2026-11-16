use super::CommandState;

impl CommandState {
    pub fn pause_hold_entries(&self) -> impl Iterator<Item = (&str, i64)> + '_ {
        self.pause_hold.iter().map(|(k, v)| (k.as_ref(), *v as i64))
    }

    pub fn active_names(&self) -> impl Iterator<Item = &str> + '_ {
        self.active.iter().map(|k| k.as_ref())
    }
}
