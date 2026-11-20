use super::*;

#[test]
fn a_download_worker_that_dies_does_not_leave_the_status_stuck() {
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    drop(tx);
    let mut updater = Updater {
        rx: Some(rx),
        state: State::Downloading {
            version: "0.4.0".into(),
            done: 3,
            total: 10,
        },
        prompt: true,
        manual: false,
    };
    assert!(matches!(updater.poll(), State::Failed(_)));
}

#[test]
fn a_check_worker_that_dies_goes_back_to_idle() {
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    drop(tx);
    let mut updater = Updater {
        rx: Some(rx),
        state: State::Checking,
        prompt: true,
        manual: false,
    };
    assert_eq!(updater.poll(), &State::Idle);
}

#[test]
fn poll_keeps_channel_open_across_empty_frames_until_found() {
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let mut updater = Updater {
        rx: Some(rx),
        state: State::Checking,
        prompt: true,
        manual: false,
    };
    assert_eq!(updater.poll(), &State::Checking);
    assert_eq!(updater.poll(), &State::Checking);
    let found = super::super::Update {
        version: "9.9.9".into(),
        notes: String::new(),
        binary: None,
    };
    tx.send(Event::Found(found.clone())).unwrap();
    assert_eq!(updater.poll(), &State::Available(found));
    assert!(updater.rx.is_none());
}

#[test]
fn poll_keeps_channel_open_during_download_progress_and_ready() {
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    let mut updater = Updater {
        rx: Some(rx),
        state: State::Downloading {
            version: "1.0.0".into(),
            done: 0,
            total: 100,
        },
        prompt: true,
        manual: false,
    };
    assert_eq!(
        updater.poll(),
        &State::Downloading {
            version: "1.0.0".into(),
            done: 0,
            total: 100,
        }
    );
    tx.send(Event::Progress { done: 50, total: 100 }).unwrap();
    assert_eq!(
        updater.poll(),
        &State::Downloading {
            version: "1.0.0".into(),
            done: 50,
            total: 100,
        }
    );
    let staged = PathBuf::from(".update-v1.0.0");
    tx.send(Event::Ready {
        version: "1.0.0".into(),
        staged: staged.clone(),
    })
    .unwrap();
    assert_eq!(
        updater.poll(),
        &State::Ready {
            version: "1.0.0".into(),
            staged,
        }
    );
    assert!(updater.rx.is_none());
}

#[test]
fn manual_check_reports_up_to_date_while_boot_check_returns_to_idle() {
    let (tx1, rx1) = std::sync::mpsc::channel::<Event>();
    let mut boot = Updater {
        rx: Some(rx1),
        state: State::Checking,
        prompt: true,
        manual: false,
    };
    tx1.send(Event::UpToDate).unwrap();
    assert_eq!(boot.poll(), &State::Idle);

    let (tx2, rx2) = std::sync::mpsc::channel::<Event>();
    let mut manual = Updater {
        rx: Some(rx2),
        state: State::Checking,
        prompt: true,
        manual: true,
    };
    tx2.send(Event::UpToDate).unwrap();
    assert_eq!(manual.poll(), &State::UpToDate);
}
