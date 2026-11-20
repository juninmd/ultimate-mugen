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
    };
    assert_eq!(updater.poll(), &State::Idle);
}
