#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    None,
}

impl PieceType {
    pub fn to_char(piece_type: PieceType) -> char {
        match piece_type {
            Self::King => 'k',
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Pawn => 'p',
            Self::None => ' ',
        }
    }
}