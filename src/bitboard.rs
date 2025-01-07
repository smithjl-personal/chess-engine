use crate::constants;

#[derive(Clone)]
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
}

pub const NOT_FILE_A: u64 = 18374403900871474942;
pub const NOT_FILE_B: u64 = 18302063728033398269;
pub const NOT_FILE_AB: u64 = 18229723555195321596;
pub const NOT_FILE_G: u64 = 13816973012072644543;
pub const NOT_FILE_H: u64 = 9187201950435737471;
pub const NOT_FILE_GH: u64 = 4557430888798830399;

pub const NOT_RANK_8: u64 = 18446744073709551360;
pub const NOT_RANK_7: u64 = 18446744073709486335;
pub const NOT_RANK_2: u64 = 18374967954648334335;
pub const NOT_RANK_1: u64 = 72057594037927935;

/*
    All Squares: 18446744073709551615

*/

pub enum CastleSides {
    Short,
    Long,
}

pub struct Move {
    pub from: usize,
    pub to: usize,

    pub is_capture: Option<bool>,
    pub is_check: Option<bool>,
    pub next_en_pessant_target_coord: Option<usize>,
    pub pawn_promoting_to: Option<PieceType>,
    pub castle_side: Option<CastleSides>,
}

impl Move {
    pub fn move_to_str(&self) -> String {
        let extra_char: String = match self.pawn_promoting_to {
            Some(t) => t.to_char_side_agnostic().to_string(),
            None => String::from(""),
        };
        return format!("{}{}{}", square_to_coord(self.from), square_to_coord(self.to), extra_char);
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

        let m = Move {
            from: from,
            to: to,
            pawn_promoting_to: pawn_promoting_to,

            // Unknown from this position. We need the bitboards to find this.
            is_capture: None,
            is_check: None,
            next_en_pessant_target_coord: None,
            castle_side: None,
        };

        return Ok(m);
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


pub enum SliderPieces {
    Queen,
    Rook,
    Bishop,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
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
}

pub struct Constants {
    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],
    pub bishop_attacks: Vec<Vec<u64>>, // [64][512]
    pub rook_attacks: Vec<Vec<u64>>,   // [64][4096]
}

impl Constants {
    pub fn new() -> Self {
        let mut pawn_attacks: [[u64; 64]; 2] = [[0; 64]; 2];
        let mut knight_attacks: [u64; 64] = [0; 64];
        let mut king_attacks: [u64; 64] = [0; 64];

        // These are too big to put on the stack.
        let mut bishop_attacks: Vec<Vec<u64>> = vec![vec![0; 512]; 64];
        let mut rook_attacks: Vec<Vec<u64>> = vec![vec![0; 4096]; 64];

        for square in 0..64 {
            pawn_attacks[Color::White.idx()][square] =
                mask_pawn_attacks(square, Color::White);
            pawn_attacks[Color::Black.idx()][square] =
                mask_pawn_attacks(square, Color::Black);

            knight_attacks[square] = mask_knight_attacks(square);
            king_attacks[square] = mask_king_attacks(square);
        }

        init_slider_attacks(true, &mut bishop_attacks, &mut rook_attacks);
        init_slider_attacks(false, &mut bishop_attacks, &mut rook_attacks);

        return Constants {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop_attacks,
            rook_attacks,
        };
    }
}

// TODO: Research more on lifetime stuff.
pub struct ChessGame<'a> {
    pub bitboard_constants: &'a Constants,

    // En-Passant
    pub en_passant_target: Option<usize>,

    // Flags.
    pub white_to_move: bool,
    pub can_white_castle_long: bool,
    pub can_white_castle_short: bool,
    pub can_black_castle_long: bool,
    pub can_black_castle_short: bool,

    // White Pieces (bitboards)
    pub white_pawns: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_knights: u64,
    pub white_queens: u64,
    pub white_king: u64,

    // Black Pieces (bitboards)
    pub black_pawns: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_knights: u64,
    pub black_queens: u64,
    pub black_king: u64,

    // Occupancies (bitboards)
    pub white_occupancies: u64,
    pub black_occupancies: u64,
    pub all_occupancies: u64,
}

impl<'a> ChessGame<'a> {
    pub fn new(c: &'a Constants) -> Self {
        return ChessGame {
            bitboard_constants: c,

            en_passant_target: None,

            white_to_move: true,
            can_white_castle_long: true,
            can_white_castle_short: true,
            can_black_castle_long: true,
            can_black_castle_short: true,

            white_pawns: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_knights: 0,
            white_queens: 0,
            white_king: 0,

            black_pawns: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_knights: 0,
            black_queens: 0,
            black_king: 0,

            white_occupancies: 0,
            black_occupancies: 0,
            all_occupancies: 0,
        };
    }

    // Takes all pieces off the board.
    pub fn clear_board(&mut self) {
        self.white_pawns = 0;
        self.white_bishops = 0;
        self.white_rooks = 0;
        self.white_knights = 0;
        self.white_queens = 0;
        self.white_king = 0;

        self.black_pawns = 0;
        self.black_bishops = 0;
        self.black_rooks = 0;
        self.black_knights = 0;
        self.black_queens = 0;
        self.black_king = 0;

        self.white_occupancies = 0;
        self.black_occupancies = 0;
        self.all_occupancies = 0;
    }

    pub fn print_board(&self) {
        // do something...
        println!("    A   B   C   D   E   F   G   H");
        println!("  |---|---|---|---|---|---|---|---|");
        for rank in 0..8 {
            print!("{} |", 8 - rank);
            for file in 0..8 {
                let square: usize = rank * 8 + file;

                let (piece_type, color) = self.get_piece_at_square(square);
                let c = match piece_type {
                    Some(t) => t.to_char(color.unwrap()),
                    None => ' ',
                };

                print!(" {} |", c);
            }
            print!(" {}", 8 - rank);
            println!();
            println!("  |---|---|---|---|---|---|---|---|");
        }
        println!("    A   B   C   D   E   F   G   H");
    }

    pub fn import_fen(&mut self, fen: &str) -> Result<(), String> {
        // Clear the board.
        self.clear_board();

        // Trim the string.
        let trimmed_full_fen = fen.trim();

        // Split by first space to separate board position from the rest of the FEN components.
        let mut parts = trimmed_full_fen.split(' ');
        let board_str = match parts.next() {
            Some(s) => s,
            None => return Err("No board position found in FEN.".to_string()),
        };

        // Prepare to populate our board.
        let rows = board_str.split('/');
        let mut y_pos: usize = 0;

        // Parse each row of the board.
        for row in rows {
            // For each char in the row, we will either have a character or a number.
            let mut x_pos: usize = 0;
            for c in row.chars() {
                // Handles empty spaces on the board.
                if c.is_digit(10) {
                    let num_empties: usize = match c.to_digit(10) {
                        Some(n) => n as usize,
                        None => return Err(format!("Failed to parse digit: {}.", c)),
                    };

                    if num_empties < 1 || num_empties > 8 {
                        return Err(format!("Invalid number of empty spaces: {}", num_empties));
                    }

                    x_pos += num_empties;
                    continue;
                }

                // Place the piece on the board.
                let piece_color: Color;
                if c.is_ascii_uppercase() {
                    piece_color = Color::White;
                } else {
                    piece_color = Color::Black;
                }

                let piece_type = match c.to_ascii_lowercase() {
                    'k' => PieceType::King,
                    'q' => PieceType::Queen,
                    'r' => PieceType::Rook,
                    'b' => PieceType::Bishop,
                    'n' => PieceType::Knight,
                    'p' => PieceType::Pawn,
                    _ => return Err(format!("Unexpected piece letter {}", c)),
                };

                let square: usize = y_pos * 8 + x_pos;
                self.place_piece_on_board(piece_color, piece_type, square);

                x_pos += 1;
            }

            // Ensure that the board has exactly 8 cols.
            if x_pos != 8 {
                return Err(format!(
                    "Board must have exactly 8 columns. We parsed: {}",
                    x_pos
                ));
            }

            y_pos += 1;
        }

        // Ensure that the board has exactly 8 rows.
        if y_pos != 8 {
            return Err(format!(
                "Board must have exactly 8 rows. We parsed: {}",
                y_pos
            ));
        }

        // Store whose turn it is to move.
        let whose_turn = match parts.next() {
            Some(s) => s,
            None => return Err("Unsure whose turn it is. Cannot proceed.".to_string()),
        };

        if whose_turn.to_ascii_lowercase() == "w" {
            self.white_to_move = true;
        } else if whose_turn.to_ascii_lowercase() == "b" {
            self.white_to_move = false;
        } else {
            return Err(format!(
                "Unexpected character for whose turn it is: {}. Should be 'w' or 'b'.",
                whose_turn
            ));
        }

        // Castling.
        let castling_rights_str = parts.next();
        match castling_rights_str {
            Some(s) => {
                // Assume no one can castle.
                self.can_white_castle_long = false;
                self.can_white_castle_short = false;
                self.can_black_castle_long = false;
                self.can_black_castle_short = false;

                // Update rights based on what we find in the string.
                for c in s.chars() {
                    match c {
                        'K' => self.can_white_castle_short = true,
                        'Q' => self.can_white_castle_long = true,
                        'k' => self.can_black_castle_short = true,
                        'q' => self.can_black_castle_long = true,
                        _ => (),
                    }
                }
            }
            None => return Ok(()),
        }

        // En-Passant target.
        let en_passant_target_str = parts.next();
        match en_passant_target_str {
            Some(s) => {
                // Try to parse the string as a coordinate.
                let parsed_coord = str_coord_to_square(s);
                if parsed_coord.is_ok() {
                    self.en_passant_target = Some(parsed_coord.unwrap());
                } else {
                    self.en_passant_target = None;
                }
            }
            None => return Ok(()),
        };

        // Half-Moves since last pawn move and capture (for 50 move rule).
        // let half_move_str = parts.next();
        // match half_move_str {
        //     Some(s) => {
        //         let parsed_number: Result<u32, _> = s.parse();
        //         if parsed_number.is_ok() {
        //             self.half_move_count_non_pawn_non_capture = parsed_number.unwrap();
        //         }
        //     }
        //     None => return,
        // }

        // Full move count. Incremented after black moves.
        // let full_move_count_str = parts.next();
        // match full_move_count_str {
        //     Some(s) => {
        //         let parsed_number: Result<u32, _> = s.parse();
        //         if parsed_number.is_ok() {
        //             self.full_move_count = parsed_number.unwrap();
        //         }
        //     }
        //     None => return,
        // }

        return Ok(());
    }

    pub fn place_piece_on_board(&mut self, side: Color, piece_type: PieceType, square: usize) {
        match side {
            Color::White => {
                match piece_type {
                    PieceType::Pawn => self.white_pawns = set_bit(self.white_pawns, square),
                    PieceType::Bishop => self.white_bishops = set_bit(self.white_bishops, square),
                    PieceType::Knight => self.white_knights = set_bit(self.white_knights, square),
                    PieceType::Rook => self.white_rooks = set_bit(self.white_rooks, square),
                    PieceType::Queen => self.white_queens = set_bit(self.white_queens, square),
                    PieceType::King => self.white_king = set_bit(self.white_king, square),
                };
                self.white_occupancies = set_bit(self.white_occupancies, square);
            }
            Color::Black => {
                match piece_type {
                    PieceType::Pawn => self.black_pawns = set_bit(self.black_pawns, square),
                    PieceType::Bishop => self.black_bishops = set_bit(self.black_bishops, square),
                    PieceType::Knight => self.black_knights = set_bit(self.black_knights, square),
                    PieceType::Rook => self.black_rooks = set_bit(self.black_rooks, square),
                    PieceType::Queen => self.black_queens = set_bit(self.black_queens, square),
                    PieceType::King => self.black_king = set_bit(self.black_king, square),
                };
                self.black_occupancies = set_bit(self.black_occupancies, square);
            }
        };

        self.all_occupancies = set_bit(self.all_occupancies, square);
    }

    // WARNING: Not efficient function??
    pub fn get_piece_at_square(&self, square: usize) -> (Option<PieceType>, Option<Color>) {
        let is_occupied = get_bit(self.all_occupancies, square) != 0;
        if !is_occupied {
            return (None, None);
        }

        let is_occupied_white = get_bit(self.white_occupancies, square) != 0;
        let is_occupied_black = get_bit(self.black_occupancies, square) != 0;

        if is_occupied_white {
            if get_bit(self.white_pawns, square) != 0 {
                return (Some(PieceType::Pawn), Some(Color::White));
            } else if get_bit(self.white_bishops, square) != 0 {
                return (Some(PieceType::Bishop), Some(Color::White));
            } else if get_bit(self.white_knights, square) != 0 {
                return (Some(PieceType::Knight), Some(Color::White));
            } else if get_bit(self.white_rooks, square) != 0 {
                return (Some(PieceType::Rook), Some(Color::White));
            } else if get_bit(self.white_queens, square) != 0 {
                return (Some(PieceType::Queen), Some(Color::White));
            } else if get_bit(self.white_king, square) != 0 {
                return (Some(PieceType::King), Some(Color::White));
            } else {
                panic!("Something has gone very wrong.");
            }
        } else if is_occupied_black {
            if get_bit(self.black_pawns, square) != 0 {
                return (Some(PieceType::Pawn), Some(Color::Black));
            } else if get_bit(self.black_bishops, square) != 0 {
                return (Some(PieceType::Bishop), Some(Color::Black));
            } else if get_bit(self.black_knights, square) != 0 {
                return (Some(PieceType::Knight), Some(Color::Black));
            } else if get_bit(self.black_rooks, square) != 0 {
                return (Some(PieceType::Rook), Some(Color::Black));
            } else if get_bit(self.black_queens, square) != 0 {
                return (Some(PieceType::Queen), Some(Color::Black));
            } else if get_bit(self.black_king, square) != 0 {
                return (Some(PieceType::King), Some(Color::Black));
            } else {
                panic!("Something has gone very wrong.");
            }
        } else {
            panic!("Someting has gone very wrong.");
        }
    }

    // Bitwise operations make this pretty quick.
    pub fn is_square_attacked(&self, square: usize, who_is_attacking: &Color) -> bool {
        match who_is_attacking {
            Color::White => {
                // Pawns.
                if self.bitboard_constants.pawn_attacks[Color::Black.idx()][square]
                    & self.white_pawns
                    != 0
                {
                    return true;
                }

                // Knights.
                if self.bitboard_constants.knight_attacks[square] & self.white_knights != 0 {
                    return true;
                }

                // Bishops.
                if self.get_bishop_attacks(square, self.all_occupancies) & self.white_bishops != 0 {
                    return true;
                }

                // Rooks.
                if self.get_rook_attacks(square, self.all_occupancies) & self.white_rooks != 0 {
                    return true;
                }

                // Queens. (we could speed this up slightly... look here for optimization if needed.)
                if self.get_queen_attacks(square, self.all_occupancies) & self.white_queens != 0 {
                    return true;
                }

                // King.
                if self.bitboard_constants.king_attacks[square] & self.white_king != 0 {
                    return true;
                }
            }
            Color::Black => {
                // Pawns.
                if self.bitboard_constants.pawn_attacks[Color::White.idx()][square]
                    & self.black_pawns
                    != 0
                {
                    return true;
                }

                // Knights.
                if self.bitboard_constants.knight_attacks[square] & self.black_knights != 0 {
                    return true;
                }

                // Bishops.
                if self.get_bishop_attacks(square, self.all_occupancies) & self.black_bishops != 0 {
                    return true;
                }

                // Rooks.
                if self.get_rook_attacks(square, self.all_occupancies) & self.black_rooks != 0 {
                    return true;
                }

                // Queens. (we could speed this up slightly... look here for optimization if needed.)
                if self.get_queen_attacks(square, self.all_occupancies) & self.black_queens != 0 {
                    return true;
                }

                // King.
                if self.bitboard_constants.king_attacks[square] & self.black_king != 0 {
                    return true;
                }
            }
        };

        // No attacks found!
        return false;
    }

    pub fn get_bishop_attacks(&self, square: usize, mut occupancy: u64) -> u64 {
        occupancy &= constants::BISHOP_MASKED_ATTACKS[square];
        (occupancy, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
        occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[square];

        return self.bitboard_constants.bishop_attacks[square][occupancy as usize];
    }

    pub fn get_rook_attacks(&self, square: usize, mut occupancy: u64) -> u64 {
        occupancy &= constants::ROOK_MASKED_ATTACKS[square];
        (occupancy, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
        occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[square];

        return self.bitboard_constants.rook_attacks[square][occupancy as usize];
    }

    pub fn get_queen_attacks(&self, square: usize, occupancy: u64) -> u64 {
        return self.get_bishop_attacks(square, occupancy)
            | self.get_rook_attacks(square, occupancy);
    }

    // Will generate moves that put self in check.
    pub fn get_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> =  vec![];

        // Get all the moves.
        moves.append(&mut self.get_moves_slider(SliderPieces::Queen));
        moves.append(&mut self.get_moves_slider(SliderPieces::Rook));
        moves.append(&mut self.get_moves_slider(SliderPieces::Bishop));
        moves.append(&mut self.get_moves_knight());
        moves.append(&mut self.get_moves_king());
        moves.append(&mut self.get_moves_pawns());

        return moves;
    }

    pub fn print_legal_moves(&self) {
        let moves = self.get_psuedo_legal_moves();
        for m in moves.iter() {
            println!("{}", m.move_to_str());
        }
    }

    pub fn get_moves_slider(&self, slider_piece_type: SliderPieces) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let mut source_square: usize;
        let mut target_square: usize;
        let mut slider_pieces: u64;
        let mut slider_piece_attacks: u64;
        let mut quiet_moves: u64;
        let mut captures: u64;
        let their_occupancies: u64;

        if self.white_to_move {
            their_occupancies = self.black_occupancies;
            match slider_piece_type {
                SliderPieces::Queen => {
                    slider_pieces = self.white_queens;
                }
                SliderPieces::Rook => {
                    slider_pieces = self.white_rooks;
                }
                SliderPieces::Bishop => {
                    slider_pieces = self.white_bishops;
                }
            }
        } else {
            their_occupancies = self.white_occupancies;
            match slider_piece_type {
                SliderPieces::Queen => {
                    slider_pieces = self.black_queens;
                }
                SliderPieces::Rook => {
                    slider_pieces = self.black_rooks;
                }
                SliderPieces::Bishop => {
                    slider_pieces = self.black_bishops;
                }
            }
        }

        while slider_pieces != 0 {
            source_square = get_lsb_index(slider_pieces).expect("This should not happen.");

            // Get moves and captures seperately.
            slider_piece_attacks = match slider_piece_type {
                SliderPieces::Queen => {
                    self.get_queen_attacks(source_square, self.all_occupancies)
                }
                SliderPieces::Rook => {
                    self.get_rook_attacks(source_square, self.all_occupancies)
                }
                SliderPieces::Bishop => {
                    self.get_bishop_attacks(source_square, self.all_occupancies)
                }
            };

            quiet_moves = slider_piece_attacks & (!self.all_occupancies);
            captures = slider_piece_attacks & their_occupancies;

            while quiet_moves != 0 {
                target_square = get_lsb_index(quiet_moves).expect("This should not be empty.");
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(false),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                });
                quiet_moves = pop_bit(quiet_moves, target_square);
            }

            while captures != 0 {
                target_square = get_lsb_index(captures).expect("This should not be empty.");
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(true),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                });
                captures = pop_bit(captures, target_square);
            }

            slider_pieces = pop_bit(slider_pieces, source_square);
        }

        return moves;
    }

    pub fn get_moves_knight(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let mut source_square: usize;
        let mut target_square: usize;
        let mut knights: u64;
        let mut quiet_moves: u64;
        let mut captures: u64;
        let their_occupancies: u64;

        if self.white_to_move {
            their_occupancies = self.black_occupancies;
            knights = self.white_knights;
        } else {
            their_occupancies = self.white_occupancies;
            knights = self.black_knights;
        }

        while knights != 0 {
            source_square = get_lsb_index(knights).expect("This should not happen.");

            // Get moves and captures seperately.
            quiet_moves = self.bitboard_constants.knight_attacks[source_square] & (!self.all_occupancies);
            captures = self.bitboard_constants.knight_attacks[source_square] & their_occupancies;

            while quiet_moves != 0 {
                target_square = get_lsb_index(quiet_moves).expect("This should not be empty.");
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(false),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                });
                quiet_moves = pop_bit(quiet_moves, target_square);
            }

            while captures != 0 {
                target_square = get_lsb_index(captures).expect("This should not be empty.");
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(true),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                });
                captures = pop_bit(captures, target_square);
            }

            knights = pop_bit(knights, source_square);
        }

        return moves;
    }

    pub fn get_moves_king(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let source_square: usize;
        let mut target_square: usize;
        let bitboard: u64;

        let their_color: &Color;
        let their_occupancies: u64;
        let can_castle_long: bool;
        let can_castle_short: bool;
        let king_starting_square: usize;
        if self.white_to_move {
            their_color = &Color::Black;
            their_occupancies = self.black_occupancies;
            bitboard = self.white_king;
            can_castle_short = self.can_white_castle_short;
            can_castle_long = self.can_white_castle_long;
            king_starting_square = 60;
        } else {
            their_color = &Color::White;
            their_occupancies = self.white_occupancies;
            bitboard = self.black_king;
            can_castle_short = self.can_black_castle_short;
            can_castle_long = self.can_black_castle_long;
            king_starting_square = 4;
        }

        if bitboard == 0 {
            return moves;
        }

        source_square = get_lsb_index(bitboard).expect("Guard before should handle this.");
        let mut quiet_moves = self.bitboard_constants.king_attacks[source_square] & (!self.all_occupancies);
        let mut attacks = self.bitboard_constants.king_attacks[source_square] & their_occupancies;


        // Moves
        while quiet_moves != 0 {
            target_square = get_lsb_index(quiet_moves).expect("Guard before should handle this.");
            moves.push(Move {
                from: source_square,
                to: target_square,
                is_capture: Some(false),
                is_check: None,
                next_en_pessant_target_coord: None,
                pawn_promoting_to: None,
                castle_side: None,
            });
            quiet_moves = pop_bit(quiet_moves, target_square);
        }

        // Attacks
        while attacks != 0 {
            target_square = get_lsb_index(attacks).expect("Guard before should handle this.");
            moves.push(Move {
                from: source_square,
                to: target_square,
                is_capture: Some(true),
                is_check: None,
                next_en_pessant_target_coord: None,
                pawn_promoting_to: None,
                castle_side: None,
            });
            attacks = pop_bit(attacks, target_square);
        }

        // Castling
        if can_castle_short {

            // 1. Make sure squares are empty.
            let squares_should_be_empty = set_bit(0, king_starting_square + 1) | set_bit(0, king_starting_square + 2);

            // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
            let is_intermediary_square_attacked = self.is_square_attacked(king_starting_square + 1, their_color);

            // If both conditions are met, we can castle.
            target_square = king_starting_square + 2;
            if (squares_should_be_empty & self.all_occupancies) == 0 && !is_intermediary_square_attacked {
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(false),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: Some(CastleSides::Short),
                });
            }
        }

        if can_castle_long {

            // 1. Make sure squares are empty.
            let squares_should_be_empty = set_bit(0, king_starting_square - 1) | set_bit(0, king_starting_square - 2) | set_bit(0, king_starting_square - 3);

            // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
            let is_intermediary_square_attacked = self.is_square_attacked(king_starting_square - 1, their_color);

            // If both conditions are met, we can castle.
            target_square = king_starting_square - 2;
            if (squares_should_be_empty & self.all_occupancies) == 0 && !is_intermediary_square_attacked {
                moves.push(Move {
                    from: source_square,
                    to: target_square,
                    is_capture: Some(false),
                    is_check: None,
                    next_en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: Some(CastleSides::Long),
                });
            }
        }

        return moves;
    }

    pub fn get_moves_pawns(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let mut source_square: usize;
        let mut target_square: usize;

        let mut bitboard: u64;
        let mut attacks: u64;

        let our_color: Color;
        let their_occupancies: u64;
        let pawn_move_offset: i32;
        let promotion_rank_lower: usize;
        let promotion_rank_upper: usize;
        let our_starting_rank_lower: usize;
        let our_starting_rank_upper: usize;
        if self.white_to_move {
            our_color = Color::White;
            their_occupancies = self.black_occupancies;
            bitboard = self.white_pawns;
            pawn_move_offset = -8;
            promotion_rank_lower = 0;
            promotion_rank_upper = 7;
            our_starting_rank_lower = 48;
            our_starting_rank_upper = 55;
        } else {
            our_color = Color::Black;
            their_occupancies = self.white_occupancies;
            bitboard = self.black_pawns;
            pawn_move_offset = 8;
            promotion_rank_lower = 56;
            promotion_rank_upper = 63;
            our_starting_rank_lower = 8;
            our_starting_rank_upper = 15;
        }

        while bitboard != 0 {
            source_square = get_lsb_index(bitboard).expect("This should not fail.");

            // Handles forward moves.
            target_square = (source_square as i32 + pawn_move_offset) as usize;
            let mut is_occupied = get_bit(self.all_occupancies, target_square) != 0;
            if !is_occupied {

                // Check for promotions (no capture).
                if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(false),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Queen),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(false),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Rook),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(false),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Bishop),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(false),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Knight),
                        castle_side: None,
                    });
                    
                } else {
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(false),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: None,
                        castle_side: None,
                    });

                    // Check for the double move.
                    target_square = (source_square as i32 + pawn_move_offset + pawn_move_offset) as usize;
                    is_occupied = get_bit(self.all_occupancies, target_square) != 0;

                    // If pawn is on the 2nd rank, it can move two tiles.
                    if source_square >= our_starting_rank_lower && source_square <= our_starting_rank_upper && !is_occupied {
                        moves.push(Move {
                            from: source_square,
                            to: target_square,
                            is_capture: Some(false),
                            is_check: None,
                            next_en_pessant_target_coord: Some((source_square as i32 + pawn_move_offset) as usize),
                            pawn_promoting_to: None,
                            castle_side: None,
                        });
                    }
                }
            }

            // Handles captures (non-en-passant).
            attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square] & their_occupancies;
            while attacks != 0 {
                target_square = get_lsb_index(attacks).expect("Should not be empty.");
                if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(true),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Queen),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(true),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Rook),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(true),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Bishop),
                        castle_side: None,
                    });
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(true),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: Some(PieceType::Knight),
                        castle_side: None,
                    });
                } else {
                    moves.push(Move {
                        from: source_square,
                        to: target_square,
                        is_capture: Some(true),
                        is_check: None,
                        next_en_pessant_target_coord: None,
                        pawn_promoting_to: None,
                        castle_side: None,
                    });
                }
                attacks = pop_bit(attacks, target_square);
            }

            // Handles captures (en-passant)
            match self.en_passant_target {
                Some(s) => {
                    attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square] & set_bit(0, s);
                    if attacks != 0 {
                        moves.push(Move {
                            from: source_square,
                            to: target_square,
                            is_capture: Some(true),
                            is_check: None,
                            next_en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                            castle_side: None,
                        });
                    }
                }
                _ => ()
            }


            // Empty the board! and go next.
            bitboard = pop_bit(bitboard, source_square);
        }

        return moves;
    }
}

pub fn square_to_coord(square: usize) -> String {
    let rank = 8 - (square / 8);
    let file_number = square % 8;
    let file_char = ('a' as u8 + file_number as u8) as char;

    return format!("{}{}", file_char, rank);
}

pub fn print_bitboard(bitboard: u64) {
    println!("    A   B   C   D   E   F   G   H");
    println!("  |---|---|---|---|---|---|---|---|");
    for rank in 0..8 {
        print!("{} |", 8 - rank);
        for file in 0..8 {
            let square: usize = rank * 8 + file;
            let calc = get_bit(bitboard, square);
            let populated;
            if calc != 0 {
                populated = 1;
            } else {
                populated = 0;
            }

            print!(" {} |", populated);
        }
        print!(" {}", 8 - rank);
        println!();
        println!("  |---|---|---|---|---|---|---|---|");
    }
    println!("    A   B   C   D   E   F   G   H");
    println!("Bitboard Value: {bitboard}");
}

pub fn mask_pawn_attacks(square: usize, side: Color) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    match side {
        Color::White => {
            // Attacking top right.
            attacks |= (bitboard >> 7) & NOT_FILE_A;

            // Attacking top left.
            attacks |= (bitboard >> 9) & NOT_FILE_H;
        }
        Color::Black => {
            // Attacking bottom right.
            attacks |= (bitboard << 9) & NOT_FILE_A;

            // Attacking bottom left.
            attacks |= (bitboard << 7) & NOT_FILE_H;
        }
    }

    return attacks;
}

pub fn mask_knight_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 10) & NOT_FILE_GH; // Up 1 Left 2
    attacks |= (bitboard >> 17) & NOT_FILE_H; // Up 2 Left 1

    attacks |= (bitboard >> 6) & NOT_FILE_AB; // Up 1 Right 2
    attacks |= (bitboard >> 15) & NOT_FILE_A; // Up 2 Right 1

    attacks |= (bitboard << 6) & NOT_FILE_GH; // Down 1 Left 2
    attacks |= (bitboard << 15) & NOT_FILE_H; // Down 2 Left 1

    attacks |= (bitboard << 10) & NOT_FILE_AB; // Down 1 Right 2
    attacks |= (bitboard << 17) & NOT_FILE_A; // Down 2 Right 1

    return attacks;
}

pub fn mask_king_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 9) & NOT_FILE_H; // Up 1 Left 1
    attacks |= bitboard >> 8; // Up 1
    attacks |= (bitboard >> 7) & NOT_FILE_A; // Up 1 Right 1

    attacks |= (bitboard >> 1) & NOT_FILE_H; // Left 1
    attacks |= (bitboard << 1) & NOT_FILE_A; // Right 1

    attacks |= (bitboard << 7) & NOT_FILE_H; // Down 1 Left 1
    attacks |= bitboard << 8; // Down 1
    attacks |= (bitboard << 9) & NOT_FILE_A; // Down 1 Right 1

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_bishop_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 6 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 6 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file -= 1;
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while rank >= 1 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank >= 1 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file -= 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_bishop_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 7 && file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        // Break the loop when we would subtract underflow.
        rank += 1;
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_rook_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 6 {
        attacks |= 1 << (rank * 8 + file);
        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while file >= 1 {
        attacks |= 1 << (rank * 8 + file);
        file -= 1;
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    while rank >= 1 {
        attacks |= 1 << (rank * 8 + file);
        rank -= 1;
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 6 {
        attacks |= 1 << (rank * 8 + file);
        rank += 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_rook_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
    }

    return attacks;
}

// Should this be an enum?
pub fn str_coord_to_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!(
            "Invalid input detected. Expected 2 chars. Got: `{}`.",
            s.len()
        ));
    }

    // We know the length is 2, so we can safely unwrap here.
    let file_str = s.chars().nth(0).unwrap().to_ascii_lowercase();
    let rank_str = s.chars().nth(1).unwrap();

    // Attempt conversion for file letter.
    let file: usize = file_str as usize - 'a' as usize;
    if file >= 8 as usize {
        return Err(format!("Invalid file letter: {}", file_str));
    }

    let rank: usize = match rank_str.to_digit(10) {
        Some(n) => (8 - n).try_into().unwrap(),
        None => return Err(format!("Unable to convert `{}` to a digit.", rank_str)),
    };

    return Ok(rank * 8 + file);
}

// I don't understand how this works yet...
pub fn set_occupancies(index: usize, bits_in_mask: usize, mut attack_mask: u64) -> u64 {
    let mut occupancy: u64 = 0;

    // Loop over bit range in attack mask.
    for count in 0..bits_in_mask {
        // Get LSB of attack mask.
        let square = match get_lsb_index(attack_mask) {
            Some(v) => v,
            None => {
                panic!("Unable to set occupancies, unexpected value for `get_lsb_index`.");
            }
        };

        // Pop the bit.
        attack_mask = pop_bit(attack_mask, square);

        // Make sure occupancy is on the board.
        if index & (1 << count) != 0 {
            occupancy |= 1 << square;
        }
    }

    return occupancy;
}

pub fn init_slider_attacks(
    is_bishop: bool,
    bishop_attacks: &mut Vec<Vec<u64>>,
    rook_attacks: &mut Vec<Vec<u64>>,
) {
    for square in 0..64 {
        let attack_mask;
        if is_bishop {
            attack_mask = constants::BISHOP_MASKED_ATTACKS[square];
        } else {
            attack_mask = constants::ROOK_MASKED_ATTACKS[square];
        }

        let relevant_bits_count = count_bits(attack_mask);
        let occupancy_indicies: usize = 1 << relevant_bits_count;
        for index in 0..occupancy_indicies {
            if is_bishop {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::BISHOP_RELEVANT_BITS[square];
                bishop_attacks[square][magic_index as usize] =
                    dynamic_bishop_attacks(square as u64, occupancy);
            } else {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::ROOK_RELEVANT_BITS[square];
                rook_attacks[square][magic_index as usize] =
                    dynamic_rook_attacks(square as u64, occupancy);
            }
        }
    }
}

// TODO: Figure out how to structure code that allows this to work... Pass in the constants?
pub fn get_bishop_attacks(c: &Constants, square: usize, mut occupancy: u64) -> u64 {
    occupancy &= constants::BISHOP_MASKED_ATTACKS[square];
    (occupancy, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[square];

    return c.bishop_attacks[square][occupancy as usize];
}

pub fn get_rook_attacks(c: &Constants, square: usize, mut occupancy: u64) -> u64 {
    occupancy &= constants::ROOK_MASKED_ATTACKS[square];
    (occupancy, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[square];

    return c.rook_attacks[square][occupancy as usize];
}

pub fn get_queen_attacks(c: &Constants, square: usize, occupancy: u64) -> u64 {
    return get_bishop_attacks(c, square, occupancy) | get_rook_attacks(c, square, occupancy);
}

// Should these be macros? Or something similar?
pub fn get_bit(bitboard: u64, square: usize) -> u64 {
    return bitboard & (1 << square);
}

pub fn set_bit(bitboard: u64, square: usize) -> u64 {
    return bitboard | (1 << square);
}

pub fn pop_bit(bitboard: u64, square: usize) -> u64 {
    if get_bit(bitboard, square) != 0 {
        return bitboard ^ (1 << square);
    } else {
        return bitboard;
    }
}

pub fn count_bits(mut bitboard: u64) -> usize {
    let mut bit_count = 0;

    while bitboard != 0 {
        bit_count += 1;

        // Reset the least significant bit, once per iteration until there are no active bits left.
        bitboard &= bitboard - 1;
    }

    return bit_count;
}

pub fn get_lsb_index(bitboard: u64) -> Option<usize> {
    // Below operations will not work on bitboard of `0`.
    if bitboard == 0 {
        return None;
    }

    // Get the position of the least-significant bit, using some bit magic!
    let lsb = bitboard & !bitboard + 1;

    // Subtract `1` to populate the trailing bits.
    let populated = lsb - 1;

    return Some(count_bits(populated));
}

/*
    // This is the code we used to generate magic numbers. We don't need to run it again, but it should remain.
    struct MagicNumberHelper {
        pub state: u32,
    }

    impl MagicNumberHelper {
        pub fn new() -> Self {
            return MagicNumberHelper {
                state: 1804289383, // Seed for our Psuedo-RNG.
            }
        }

        fn get_random_number_u32(&mut self) -> u32 {
            // Get current state. This is our seed.
            let mut n = self.state;

            // XOR Shift Algorithm to get a random number.
            n ^= n << 13;
            n ^= n >> 17;
            n ^= n << 5;

            // Update the state.
            self.state = n;

            return n;
        }

        fn get_random_number_u64(&mut self) -> u64 {
            // Define some random numbers. We want the 16 bits from MSB1 side.
            let n1: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n2: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n3: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n4: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;

            // Return them with fanciness.
            return n1 | (n2 << 16) | (n3 << 32) | (n4 << 48);
        }

        fn get_magic_number(&mut self) -> u64 {
            return self.get_random_number_u64() & self.get_random_number_u64() & self.get_random_number_u64();
        }

        // Magic numbers?
        fn find_magic_number(&mut self, square: u64, relevant_bits: usize, is_bishop: bool) -> u64 {

            // Init occupancies, attack tables, and used attacks.
            let mut occupancies: [u64; 4096] = [0; 4096];
            let mut attacks: [u64; 4096] = [0; 4096];
            let mut used_attacks: [u64; 4096];

            // Init attack mask, either bishop or rook.
            let attack_mask: u64;
            if is_bishop {
                attack_mask = mask_bishop_attacks(square);
            } else {
                attack_mask = mask_rook_attacks(square);
            }

            // Init occupancy indicies.
            let occupancy_indicies: usize = 1 << relevant_bits;

            // Loop over occupancy indicies.
            for index in 0..occupancy_indicies {
                occupancies[index] = set_occupancies(index, relevant_bits, attack_mask);

                if is_bishop {
                    attacks[index] = dynamic_bishop_attacks(square, occupancies[index]);
                } else {
                    attacks[index] = dynamic_rook_attacks(square, occupancies[index]);
                }
            }

            // Now test for magic numbers. Should not take too long to run though!
            for _ in 0..1_000_000_000 {
                // Generate magic number candidate.
                let magic_number = self.get_magic_number();

                // This should be safe from overflow?
                let (temp, _) = attack_mask.overflowing_mul(magic_number);
                let to_check = temp & 0xFF00000000000000;

                // Go next if we don't have enough bits.
                if count_bits(to_check) < 6 {
                    continue;
                }

                // Clear out any used attacks from previous iteration.
                used_attacks = [0; 4096];
                let mut has_failed: bool = false;
                for index in 0..occupancy_indicies {

                    // Overflow safe?
                    let (temp, _) = occupancies[index].overflowing_mul(magic_number);
                    let magic_index = (temp >> (64 - relevant_bits)) as usize;

                    if used_attacks[magic_index] == 0 {
                        used_attacks[magic_index] = attacks[index];
                    } else if used_attacks[magic_index] != attacks[index] {
                        has_failed = true;
                        break;
                    }
                }

                if !has_failed {
                    return magic_number;
                }
            }

            panic!("Unable to find magic number, oh no!");
        }

        // This function will print all the magic numbers, then they can be copied for later use in other parts of the program.
        pub fn init_magic_numbers(&mut self, bishop_magic_numbers: &mut [u64; 64], rook_magic_numbers: &mut [u64; 64]) {
            // For each square on the board.
            for square in 0..64 {
                // Handle rooks.
                let n = self.find_magic_number(square, constants::ROOK_RELEVANT_BITS[square as usize], false);
                // println!("0x{:X}ULL", n);
                rook_magic_numbers[square as usize] = n;
            }

            println!("\nXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\n");

            // For each square on the board.
            for square in 0..64 {
                // Handle bishops.
                let n = self.find_magic_number(square, constants::BISHOP_RELEVANT_BITS[square as usize], true);
                // println!("0x{:X}ULL", n);
                bishop_magic_numbers[square as usize] = n;
            }
        }
    }
*/
