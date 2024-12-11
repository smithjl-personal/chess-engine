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
    pub fn evaluation_value(&self) -> f64 {
        match self {
            Self::King => 0.0,
            Self::Queen => 9.0,
            Self::Rook => 5.0,
            Self::Bishop => 3.2,
            Self::Knight => 3.0,
            Self::Pawn => 1.0,
            Self::None => 0.0,
        }
    }

    pub fn char_to_piece_type(c: char) -> Result<PieceType, String> {
        return match c.to_ascii_lowercase() {
            'k' => Ok(PieceType::King),
            'q' => Ok(PieceType::Queen),
            'r' => Ok(PieceType::Rook),
            'b' => Ok(PieceType::Bishop),
            'n' => Ok(PieceType::Knight),
            'p' => Ok(PieceType::Pawn),
            _ => Err(format!(
                "Unexpected character. Cannot convert character `{}` to piece type.",
                c
            )),
        };
    }

    pub fn to_char(piece_type: PieceType, white: bool) -> char {
        let c = match piece_type {
            Self::King => 'k',
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Pawn => 'p',
            Self::None => ' ',
        };

        if white {
            return c.to_ascii_uppercase();
        } else {
            return c;
        }
    }
}
