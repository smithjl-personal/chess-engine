use crate::constants::BOARD_SIZE; // Importing the constant

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn str_to_coord(s: &str) -> Result<Coord, String> {
        if s.len() != 2 {
            return Err(format!(
                "Invalid input detected. Expected 2 chars. Got: `{}`.",
                s.len()
            ));
        }

        // We know the length is 2, so we can safely unwrap here.
        let file_letter = s.chars().nth(0).unwrap().to_ascii_lowercase();
        let rank_number = s.chars().nth(1).unwrap();

        // Attempt conversion for file letter.
        let x: i32 = file_letter as i32 - 'a' as i32;
        if x < 0 || x >= BOARD_SIZE as i32 {
            return Err(format!("Invalid file letter: {}", file_letter));
        }

        let rank_number_converted = match rank_number.to_digit(10) {
            Some(n) => n,
            None => return Err(format!("Unable to convert `{}` to a digit.", rank_number)),
        };

        let y: i32 = BOARD_SIZE as i32 - rank_number_converted as i32;
        if y < 0 || y >= BOARD_SIZE as i32 {
            return Err(format!(
                "Digit referenced `{}` is outside board size {}.",
                rank_number, BOARD_SIZE
            ));
        }

        let coord = Coord {
            x: x as usize,
            y: y as usize,
        };

        return Ok(coord);
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = (b'a' + self.x as u8) as char;
        let rank = BOARD_SIZE - self.y;
        write!(f, "{}{}", file, rank)
    }
}
