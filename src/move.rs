use crate::castle_sides::CastleSides;
use crate::helpers::{square_to_coord, str_coord_to_square};
use crate::piece_type::PieceType;

// Think about if this is the best way to do this...
#[derive(Copy, Clone, Debug)]
pub struct Move {
    // Basic data, required
    pub from_square: usize,
    pub from_piece_type: Option<PieceType>,

    pub to_square: usize,
    pub to_piece_type: Option<PieceType>, // If not 'None', then this is a capture.

    // En-Passant target tracking.
    pub last_en_passant_target_coord: Option<usize>,
    pub next_en_passant_target_coord: Option<usize>,
    pub is_en_passant_capture: bool,

    // Pawn promotion.
    pub pawn_promoting_to: Option<PieceType>,

    // Castling
    pub castle_side: Option<CastleSides>,

    // Populated later, used for move sorting.
    pub is_check: Option<bool>,
    pub removes_white_castling_rights_short: Option<bool>,
    pub removes_white_castling_rights_long: Option<bool>,
    pub removes_black_castling_rights_short: Option<bool>,
    pub removes_black_castling_rights_long: Option<bool>,
}

impl Move {
    pub fn new(from_square: usize, to_square: usize) -> Self {
        return Move {
            from_square,
            from_piece_type: None,
            to_square,
            to_piece_type: None,
            last_en_passant_target_coord: None,
            next_en_passant_target_coord: None,
            is_en_passant_capture: false,
            pawn_promoting_to: None,
            castle_side: None,
            removes_white_castling_rights_short: None,
            removes_white_castling_rights_long: None,
            removes_black_castling_rights_short: None,
            removes_black_castling_rights_long: None,
            is_check: None,
        };
    }

    pub fn move_to_str(&self) -> String {
        let extra_char: String = match self.pawn_promoting_to {
            Some(t) => t.to_char_side_agnostic().to_string(),
            None => String::from(""),
        };
        return format!(
            "{}{}{}",
            square_to_coord(self.from_square),
            square_to_coord(self.to_square),
            extra_char
        );
    }

    pub fn str_to_move(text: &str) -> Result<Move, String> {
        if text.len() != 4 && text.len() != 5 {
            return Err(format!(
                "Invalid input detected. Expected 4 or 5 chars. Got: `{}`.",
                text.len()
            ));
        }

        let from_coord = str_coord_to_square(&text[..2]);
        let to_coord = str_coord_to_square(&text[2..4]);

        let from = match from_coord {
            Ok(c) => c,
            Err(msg) => return Err(msg),
        };
        let to = match to_coord {
            Ok(c) => c,
            Err(msg) => return Err(msg),
        };

        let mut pawn_promoting_to: Option<PieceType> = None;
        if text.len() == 5 {
            let promotion_char: char = text.chars().nth(4).unwrap();
            let parsed_promotion_piece_type: Result<PieceType, String> =
                PieceType::char_to_piece_type(promotion_char);
            match parsed_promotion_piece_type {
                Ok(t) => pawn_promoting_to = Some(t),
                Err(m) => return Err(m),
            };
        }

        let mut m = Move::new(from, to);
        m.pawn_promoting_to = pawn_promoting_to;

        return Ok(m);
    }
}

// When comparing moves, we only care about the `from` and `to` and promotion. The other fields are for other parts of the program.
impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        return self.from_square == other.from_square
            && self.to_square == other.to_square
            && self.pawn_promoting_to == other.pawn_promoting_to;
    }
}
