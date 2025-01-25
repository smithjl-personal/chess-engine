use crate::constants;
use crate::color::Color;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    pub fn piece_base_value(&self) -> i64 {
        return match self {
            Self::King => 0,
            Self::Queen => 900,
            Self::Rook => 500,
            Self::Bishop => 320,
            Self::Knight => 300,
            Self::Pawn => 100,
        };
    }

    pub fn piece_happy_square_value(&self, square: usize, is_white_piece: bool) -> i64 {
        return match self {
            Self::King => constants::KING_HAPPY_SQUARES_NON_ENDGAME[square],
            Self::Queen => constants::QUEEN_HAPPY_SQUARES[square],
            Self::Rook => constants::ROOK_HAPPY_SQUARES[square],
            Self::Bishop => constants::BISHOP_HAPPY_SQUARES[square],
            Self::Knight => constants::KNIGHT_HAPPY_SQUARES[square],
            Self::Pawn => {
                if is_white_piece {
                    return constants::PAWN_HAPPY_SQUARES[square];
                } else {
                    // Break the square into it's x and y components; and negate the y component.
                    let rank: usize = 7 - (square / 8);
                    let file_number: usize = square % 8;

                    // Rebuild the coordinate.
                    let new_square: usize = rank * 8 + file_number;

                    // Get the value.
                    return constants::PAWN_HAPPY_SQUARES[new_square];
                }
            },
        };
    }

    pub fn to_char_side_agnostic(&self) -> char {
        return match self {
            Self::King => 'k',
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Pawn => 'p',
        };
    }

    pub fn to_char(&self, side: Color) -> char {
        let c = self.to_char_side_agnostic();

        return match side {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        };
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

    pub fn bitboard_index(&self) -> usize {
        return match self {
            Self::Pawn => 0,
            Self::Bishop => 1,
            Self::Knight => 2,
            Self::Rook => 3,
            Self::Queen => 4,
            Self::King => 5,
        };
    }

    pub fn bitboard_index_to_piece_type(i: usize) -> Self {
        return match i % 6 {
            0 => Self::Pawn,
            1 => Self::Bishop,
            2 => Self::Knight,
            3 => Self::Rook,
            4 => Self::Queen,
            5 => Self::King,
            _ => {
                panic!("Something has gone wrong converting bitboard index to piece type.");
            },
        };
    }
}
