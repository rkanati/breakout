
#[derive(Clone, Copy, Debug)]
pub struct Scoring {
    pub score:            i64,
    pub combo_score:      i64,
    pub combo_multiplier: f64,
    pub combo_max:        i64,
    pub penalties:        i64,
}

impl Scoring {
    pub fn new() -> Scoring {
        Scoring {
            score:            0,
            combo_score:      0,
            combo_multiplier: 1.,
            combo_max:        0,
            penalties:        0,
        }
    }

    fn end_combo(&mut self) -> i64 {
        let combo = self.combo_score;
        self.combo_score = 0;
        self.combo_max = self.combo_max.max(combo);
        self.combo_multiplier = 1.;
        combo
    }

    pub fn hit_floor(&mut self) {
        let combo = self.end_combo();
        self.score     -= combo;
        self.penalties += combo;
    }

    pub fn hit_paddle(&mut self) {
        let combo = self.end_combo();
        self.score += combo;
    }

    pub fn block_broken(&mut self, block_score: i64) {
        let score = self.combo_multiplier * block_score as f64;
        self.combo_score += score.round() as i64;
        self.combo_multiplier += 1.;
    }

    pub fn block_damaged(&mut self) {
        self.combo_multiplier += 0.1;
    }

    pub fn no_combo(&self) -> bool {
        self.combo_score == 0
    }

    pub fn rank(&self) -> Rank {
        match ((self.penalties as f64 / self.score as f64) * 1000.).trunc() as i64 {
                0 ..=   5 => Rank::S,
                6 ..=  25 => Rank::A,
                26 ..=  50 => Rank::B,
                51 ..= 100 => Rank::C,
            101 ..= 150 => Rank::D,
            151 ..= 250 => Rank::E,
            _           => Rank::F
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    F = 0,
    E = 1,
    D = 2,
    C = 3,
    B = 4,
    A = 5,
    S = 6,
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let string = match *self {
            Rank::F => "F",
            Rank::E => "E",
            Rank::D => "D",
            Rank::C => "C",
            Rank::B => "B",
            Rank::A => "A",
            Rank::S => "S",
        };
        f.write_str(string)
    }
}

