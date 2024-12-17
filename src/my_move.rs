use crate::{coord::Coord, piece_type::PieceType};

#[derive(Debug, Copy, Clone)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub is_capture: Option<bool>,

    pub en_pessant_target_coord: Option<Coord>, // Does this need to be part of this?
    pub pawn_promoting_to: Option<PieceType>,
}

impl Move {
    pub fn move_to_str(&self) -> String {
        let extra_char: String = match self.pawn_promoting_to {
            Some(t) => PieceType::to_char(t, false).to_string(),
            None => String::from(""),
        };
        return format!("{}{}{}", self.from, self.to, extra_char);
    }

    pub fn str_to_move(text: &str) -> Result<Move, String> {
        if text.len() != 4 && text.len() != 5 {
            return Err(format!(
                "Invalid input detected. Expected 4 or 5 chars. Got: `{}`.",
                text.len()
            ));
        }

        let from_coord = Coord::str_to_coord(&text[..2]);
        let to_coord = Coord::str_to_coord(&text[2..4]);

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

        let m = Move {
            from: from,
            to: to,
            pawn_promoting_to: pawn_promoting_to,

            // We don't know this data without the board.
            is_capture: None,
            en_pessant_target_coord: None,
        };

        return Ok(m);
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Code smells here...
        let extra_char: String = match self.pawn_promoting_to {
            Some(t) => PieceType::to_char(t, false).to_string(),
            None => String::from(""),
        };
        write!(f, "{}{}{}", self.from, self.to, extra_char)
    }
}

// When comparing moves, we only care about the `from` and `to` and promotion. The other fields are for other parts of the program.
impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        return self.from == other.from
            && self.to == other.to
            && self.pawn_promoting_to == other.pawn_promoting_to;
    }
}
