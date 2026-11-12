/// Team battle state: sequence (KOF-style turns, winner keeps its remaining life) or parallel (2v2 simultaneous).
#[derive(Clone)]
pub struct Team {
    pub members: [Vec<usize>; 2],
    pub idx: [usize; 2],
    pub life: [f32; 2],
    /** KOF XV: the team shares one super meter */
    pub power: [f32; 2],
    pub ko: [Vec<bool>; 2],
    pub simul: bool,
    pub tag: bool,
}
impl Team {
    pub fn new(a: Vec<usize>, b: Vec<usize>, simul: bool) -> Self {
        let (la, lb) = (a.len(), b.len());
        Team {
            members: [a, b],
            idx: [0, 0],
            life: [1.0, 1.0],
            power: [0.0, 0.0],
            ko: [vec![false; la], vec![false; lb]],
            simul,
            tag: false,
        }
    }
    pub fn tagged(a: Vec<usize>, b: Vec<usize>) -> Self {
        Team {
            tag: true,
            ..Team::new(a, b, false)
        }
    }
    pub fn whole_squad(&self) -> bool {
        self.simul || self.tag
    }
}
