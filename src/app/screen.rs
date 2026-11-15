pub enum Screen {
    Title,
    Menu {
        cur: usize,
    },
    Select {
        cur: [usize; 2],
        picking: usize,
        picked: [Option<usize>; 2],
    },
    Stages {
        cur: usize,
    },
    Vs {
        t: f32,
        step: u8,
    },
    Fight,
    Results {
        winner: usize,
        cur: usize,
    },
    Story {
        t: f32,
    },
    /// Survival game over: final score and fights won
    SurvivalOver {
        score: i64,
        rounds: usize,
    },
    Controls,
    Mugen {
        view: u8,
        page: usize,
    },
    Market,
    /// on-disk replay browser
    Replays {
        cur: usize,
    },
    Settings {
        tab: usize,
        cur: usize,
        from_fight: bool,
    },
    /// story scene: 0 arcade opening · 1 before the boss · 2 ending
    Cutscene {
        kind: u8,
        page: usize,
        t: f32,
    },
}
