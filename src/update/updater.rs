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
                let auto =
                    !auto_disabled(args, std::env::var("UME_NO_AUTO_UPDATE").ok().as_deref());
                let (tx, rx) = std::sync::mpsc::channel();
                let _ = std::thread::Builder::new()
                    .name("update".into())
                    .spawn(move || work(VERSION, auto, tx));
                return Updater {
                    rx: Some(rx),
                    state: State::Checking,
                };
            }
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        let _ = args;
        Updater {
            rx: None,
            state: State::Idle,
        }
    }

    pub fn poll(&mut self) -> &State {
        let Some(rx) = self.rx.take() else {
            return &self.state;
        };
        let mut keep = true;
        while keep {
            match rx.try_recv() {
                Ok(Event::Found(update)) => self.state = State::Available(update),
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
                }
                Ok(Event::Failed(error)) => {
                    self.state = State::Failed(error);
                    keep = false;
                }
                Ok(Event::UpToDate) => {
                    self.state = State::Idle;
                    keep = false;
                }
                Err(TryRecvError::Empty) => keep = false,
                Err(TryRecvError::Disconnected) => {
                    if matches!(self.state, State::Checking) {
                        self.state = State::Idle;
                    }
                    keep = false;
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
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn work(current: &'static str, auto: bool, tx: Sender<Event>) {
    let update = match download::fetch_manifest(current) {
        Ok(Some(update)) => update,
        Ok(None) => {
            let _ = tx.send(Event::UpToDate);
            return;
        }
        Err(error) => {
            let _ = tx.send(Event::Failed(error));
            return;
        }
    };
    let _ = tx.send(Event::Found(update.clone()));
    let (Some(binary), true) = (update.binary.clone(), auto) else {
        return;
    };
    let progress_tx = tx.clone();
    let mut last_bucket = u64::MAX;
    let result = download::download_and_stage(&update.version, &binary, |done, total| {
        let percent = if total > 0 {
            done.saturating_mul(100) / total
        } else {
            0
        };
        if percent / 5 != last_bucket {
            last_bucket = percent / 5;
            let _ = progress_tx.send(Event::Progress { done, total });
        }
    });
    match result {
        Ok(staged) => {
            let _ = tx.send(Event::Ready {
                version: update.version,
                staged,
            });
        }
        Err(error) => {
            let _ = tx.send(Event::Failed(error));
        }
    }
}
