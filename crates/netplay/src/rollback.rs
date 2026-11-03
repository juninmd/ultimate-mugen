use crate::checksum::checksum;
use ggrs::{Config, GgrsError, GgrsRequest, Message, NonBlockingSocket, PredictRepeatLast, SessionBuilder};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use ume_engine::matchs::{Event, Match};

pub struct Netplay<A>(PhantomData<A>);

impl<A: Clone + PartialEq + Eq + Hash + Debug + 'static> Config for Netplay<A> {
    type Input = u32;
    type InputPredictor = PredictRepeatLast;
    type State = Match;
    type Address = A;
}

pub struct Rollback {
    pub game: Match,
    pub rollbacks: u64,
    pub resimulated: u64,
    pub saves: VecDeque<(i32, u128)>,
}

const SAVE_HISTORY: usize = 256;

impl Rollback {
    pub fn new(game: Match) -> Self {
        Rollback { game, rollbacks: 0, resimulated: 0, saves: VecDeque::new() }
    }

    pub fn checksum_at(&self, frame: i32) -> Option<u128> {
        self.saves.iter().find(|(f, _)| *f == frame).map(|(_, c)| *c)
    }

    pub fn handle<A: Clone + PartialEq + Eq + Hash + Debug + 'static>(&mut self, requests: Vec<GgrsRequest<Netplay<A>>>) -> Vec<Event> {
        let advances = requests.iter().filter(|r| matches!(r, GgrsRequest::AdvanceFrame { .. })).count();
        let mut seen = 0;
        let mut events = Vec::new();
        for r in requests {
            match r {
                GgrsRequest::SaveGameState { cell, frame } => {
                    let sum = checksum(&self.game);
                    cell.save(frame, Some(self.game.clone()), Some(sum));
                    self.saves.retain(|(f, _)| *f < frame);
                    self.saves.push_back((frame, sum));
                    if self.saves.len() > SAVE_HISTORY {
                        self.saves.pop_front();
                    }
                }
                GgrsRequest::LoadGameState { cell, .. } => {
                    if let Some(m) = cell.load() {
                        self.game = m;
                        self.rollbacks += 1;
                    }
                }
                GgrsRequest::AdvanceFrame { inputs } => {
                    seen += 1;
                    let keys: Vec<u32> = inputs.iter().map(|(i, _)| *i).collect();
                    self.game.events.clear();
                    self.game.tick(&keys);
                    if seen < advances {
                        self.resimulated += 1;
                    } else {
                        events = std::mem::take(&mut self.game.events);
                    }
                }
            }
        }
        events
    }
}

pub fn run_sync_test(game: Match, inputs: &[[u32; 2]], check_distance: usize) -> Result<Rollback, GgrsError> {
    let mut session = SessionBuilder::<Netplay<usize>>::new().with_num_players(2)?.with_check_distance(check_distance).start_synctest_session()?;
    let mut rb = Rollback::new(game);
    for f in inputs {
        for (p, &k) in f.iter().enumerate() {
            session.add_local_input(p, k)?;
        }
        let requests = session.advance_frame()?;
        rb.handle(requests);
    }
    Ok(rb)
}

type Wire = Rc<RefCell<VecDeque<(u64, usize, Message)>>>;

pub struct MemorySocket {
    pub id: usize,
    pub latency: u64,
    clock: Rc<RefCell<u64>>,
    wires: Vec<Wire>,
}

impl MemorySocket {
    pub fn network(peers: usize, latency_steps: u64) -> (Vec<MemorySocket>, Rc<RefCell<u64>>) {
        let clock = Rc::new(RefCell::new(0));
        let wires: Vec<Wire> = (0..peers).map(|_| Rc::default()).collect();
        let sockets = (0..peers).map(|id| MemorySocket { id, latency: latency_steps, clock: clock.clone(), wires: wires.clone() }).collect();
        (sockets, clock)
    }
}

impl NonBlockingSocket<usize> for MemorySocket {
    fn send_to(&mut self, msg: &Message, addr: &usize) {
        let due = *self.clock.borrow() + self.latency;
        if let Some(w) = self.wires.get(*addr) {
            w.borrow_mut().push_back((due, self.id, msg.clone()));
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(usize, Message)> {
        let now = *self.clock.borrow();
        let mut wire = self.wires[self.id].borrow_mut();
        let mut out = Vec::new();
        while wire.front().is_some_and(|(due, _, _)| *due <= now) {
            let (_, from, m) = wire.pop_front().unwrap();
            out.push((from, m));
        }
        out
    }
}
