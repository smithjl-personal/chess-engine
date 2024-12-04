use crate::coord::Coord;

#[derive(Debug)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub is_capture: bool,
    // TODO: Consider storing if move is a capture, and if move is a check. Will help find good moves.
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}