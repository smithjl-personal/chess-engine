use crate::constants::BOARD_SIZE;  // Importing the constant

#[derive(Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = (b'a' + self.x as u8) as char;
        let rank = BOARD_SIZE - self.y;
        write!(f, "{}{}", file, rank)
    }
}