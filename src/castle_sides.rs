use crate::coord::Coord;

#[derive(PartialEq)]
pub enum CastleSides {
    Short,
    Long,
}

impl CastleSides {
    pub fn get_rook_start_coord(&self, white: bool) -> Coord {
        match self {
            CastleSides::Short => {
                let y: usize = if white { 7 } else { 0 };
                return Coord { x: 7, y: y };
            }
            CastleSides::Long => {
                let y: usize = if white { 7 } else { 0 };
                return Coord { x: 0, y: y };
            }
        }
    }
}
