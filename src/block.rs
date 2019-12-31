
use {
    crate::math::*,
};

pub enum Kind {
    Invlunerable,
    Scoring { score: i32, hp: i32 }
}

pub struct Block {
    pub rect: Rect,
    pub kind: Kind,
}

pub enum Hit {
    Broken(i32),
    Damaged,
    Invlunerable
}

impl Block {
    pub fn hit(&mut self) -> Hit {
        if let Kind::Scoring { score, hp } = &mut self.kind {
            *hp -= 1;
            if *hp == 0 { Hit::Broken(*score) }
            else        { Hit::Damaged }
        }
        else {
            Hit::Invlunerable
        }
    }

    pub fn hp(&self) -> Option<i32> {
        match self.kind {
            Kind::Scoring { hp, .. } => Some(hp),
            _                        => None
        }
    }

    pub fn is_scoring(&self) -> bool {
        self.hp().is_some()
    }

}
