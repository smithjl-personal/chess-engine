#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn idx(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
    pub fn piece_bitboard_offset(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 6,
        }
    }
    pub fn occupancy_bitboard_index(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}
