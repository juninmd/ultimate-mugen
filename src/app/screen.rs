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
    Tower {
        t: f32,
    },
    /// Power Arena setup: squad size, enemy count and kind
    ArenaSetup {
        row: usize,
    },
    /// Survival game over: final score and fights won
    SurvivalOver {
        score: i64,
        rounds: usize,
    },
    Controls,
    CharTuning {
        ch: usize,
        cur: usize,
    },
    Mugen {
        view: u8,
        page: usize,
    },
    Market,
    /// on-disk replay browser
    Replays {
        cur: usize,
    },
    /// local player card; tab 0 card, 1 fighters, 2 matches, 3 rivals, 4 achievements
    PlayerCard {
        tab: usize,
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
