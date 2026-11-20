use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use super::{auto_disabled, disabled_by, download, install, manifest, VERSION};
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use std::sync::mpsc::Sender;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Idle,
    Checking,
    UpToDate,
    Available(super::Update),
    Downloading {
        version: String,
        done: u64,
        total: u64,
    },
    Ready {
        version: String,
        staged: PathBuf,
    },
    Failed(String),
}

impl State {
    pub fn ready(&self) -> Option<(&str, &std::path::Path)> {
        match self {
            State::Ready { version, staged } => Some((version, staged)),
            _ => None,
        }
    }
}

enum Event {
    Found(super::Update),
    Progress { done: u64, total: u64 },
    Ready { version: String, staged: PathBuf },
    Failed(String),
    UpToDate,
}

pub struct Updater {
    rx: Option<Receiver<Event>>,
    state: State,
    prompt: bool,
    manual: bool,
}

impl Updater {
    pub fn start(args: &[String]) -> Self {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            install::cleanup_previous();
            let off = disabled_by(args, std::env::var("UME_NO_UPDATE_CHECK").ok().as_deref())
                || VERSION.ends_with("-dev")
                || manifest::platform().is_none()
                || args.iter().any(|a| a == "--shots");
            if !off {
                let prompt =
                    !auto_disabled(args, std::env::var("UME_NO_AUTO_UPDATE").ok().as_deref());
                let (tx, rx) = std::sync::mpsc::channel();
                let _ = std::thread::Builder::new()
                    .name("update".into())
                    .spawn(move || work(VERSION, tx));
                return Updater {
                    rx: Some(rx),
                    state: State::Checking,
                    prompt,
                    manual: false,
                };
            }
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        let _ = args;
        Updater {
            rx: None,
            state: State::Idle,
            prompt: false,
            manual: false,
        }
    }

    pub fn check_now(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            if matches!(
                self.state,
                State::Checking | State::Downloading { .. } | State::Ready { .. }
            ) {
                return;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            self.state = State::Checking;
            self.rx = Some(rx);
            self.manual = true;
            self.prompt = true;
            let _ = std::thread::Builder::new()
                .name("update-manual".into())
                .spawn(move || work(VERSION, tx));
        }
    }

    pub fn poll(&mut self) -> &State {
        let Some(rx) = self.rx.take() else {
            return &self.state;
        };
        let mut keep = true;
        loop {
            match rx.try_recv() {
                Ok(Event::Found(update)) => {
                    self.state = State::Available(update);
                    keep = false;
                    break;
                }
                Ok(Event::Progress { done, total }) => {
                    let version = match &self.state {
                        State::Available(update) => update.version.clone(),
                        State::Downloading { version, .. } => version.clone(),
                        _ => String::new(),
                    };
                    if !version.is_empty() {
                        self.state = State::Downloading {
                            version,
                            done,
                            total,
                        };
                    }
                }
                Ok(Event::Ready { version, staged }) => {
                    self.state = State::Ready { version, staged };
                    keep = false;
                    break;
                }
                Ok(Event::Failed(error)) => {
                    self.state = State::Failed(error);
                    keep = false;
                    break;
                }
                Ok(Event::UpToDate) => {
                    self.state = if self.manual {
                        State::UpToDate
                    } else {
                        State::Idle
                    };
                    keep = false;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if matches!(self.state, State::Checking) {
                        self.state = State::Idle;
                    } else if matches!(self.state, State::Downloading { .. }) {
                        self.state = State::Failed("download interrupted".into());
                    }
                    keep = false;
                    break;
                }
            }
        }
        if keep {
            self.rx = Some(rx);
        }
        &self.state
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        self.rx = None;
    }

    pub fn prompt_allowed(&self) -> bool {
        self.prompt
    }

    pub fn accept(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            let State::Available(update) = &self.state else {
                return;
            };
            let Some(binary) = update.binary.clone() else {
                return;
            };
            let version = update.version.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            self.state = State::Downloading { version: version.clone(), done: 0, total: binary.size };
            self.rx = Some(rx);
            let _ = std::thread::Builder::new()
                .name("update-download".into())
                .spawn(move || download_work(version, binary, tx));
        }
    }

    pub fn dismiss(&mut self) {
        if matches!(self.state, State::Available(_)) {
            self.state = State::Idle;
        }
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn work(current: &'static str, tx: Sender<Event>) {
    let event = match download::fetch_manifest(current) {
        Ok(Some(update)) => Event::Found(update),
        Ok(None) => Event::UpToDate,
        Err(error) => Event::Failed(error),
    };
    let _ = tx.send(event);
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn download_work(version: String, binary: super::manifest::Asset, tx: Sender<Event>) {
    let progress_tx = tx.clone();
    let mut last_bucket = u64::MAX;
    let result = download::download_and_stage(&version, &binary, |done, total| {
        let percent = if total > 0 { done.saturating_mul(100) / total } else { 0 };
        if percent / 5 != last_bucket {
            last_bucket = percent / 5;
            let _ = progress_tx.send(Event::Progress { done, total });
        }
    });
    let event = match result {
        Ok(staged) => Event::Ready { version, staged },
        Err(error) => Event::Failed(error),
    };
    let _ = tx.send(event);
}

#[cfg(test)]
mod tests;
