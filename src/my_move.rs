use crate::coord::Coord;

#[derive(Debug, Copy, Clone)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub is_capture: bool,
    // TODO: Consider storing if move is a capture, and if move is a check. Will help find good moves.
}

impl Move {
    pub fn str_to_move(text: &str) -> Result<Move, String> {
        if text.len() != 4 {
            return Err(format!("Invalid input detected. Expected 2 chars. Got: `{}`.", text.len()));
        }

        let from_coord = Coord::str_to_coord(&text[..2]);
        let to_coord = Coord::str_to_coord(&text[2..]);

        let from = match from_coord {
            Ok(c) => c,
            Err(msg) => return Err(msg),
        };
        let to = match to_coord {
            Ok(c) => c,
            Err(msg) => return Err(msg),
        };

        let m = Move {
            from: from,
            to: to,
            is_capture: false, // TODO: Make this a `None` option.
        };

        return Ok(m);
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}

// When comparing moves, we only care about the `from` and `to`. The other fields are for other parts of the program.
impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}