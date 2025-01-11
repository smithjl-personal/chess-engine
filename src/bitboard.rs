use crate::constants;

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


#[derive(Copy, Clone, Debug)]
pub enum CastleSides {
    Short,
    Long,
}

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

    // Pawn promotion.
    pub pawn_promoting_to: Option<PieceType>,

    // Castling
    pub castle_side: Option<CastleSides>,
    pub removes_castling_rights_short: bool,
    pub removes_castling_rights_long: bool,

    // Populated later, used for move sorting.
    pub is_check: Option<bool>,
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
            pawn_promoting_to: None,
            castle_side: None,
            removes_castling_rights_short: false,
            removes_castling_rights_long: false,
            is_check: None,
        }
    }

    pub fn move_to_str(&self) -> String {
        let extra_char: String = match self.pawn_promoting_to {
            Some(t) => t.to_char_side_agnostic().to_string(),
            None => String::from(""),
        };
        return format!("{}{}{}", square_to_coord(self.from_square), square_to_coord(self.to_square), extra_char);
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


pub enum SliderPieces {
    Queen,
    Rook,
    Bishop,
}

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
#[derive(Copy, Clone)]
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

    /*
        0 -> white_pawns
        1 -> white_bishops
        2 -> white_knights
        3 -> white_rooks
        4 -> white_queens
        5 -> white_kings
        6 -> black_pawns
        ...
    */
    pub piece_bitboards: [u64; 12],

    /*
        0 -> white_occupancies
        1 -> black_occupancies
        2 -> all_occupancies
    */
    pub occupancy_bitboards: [u64; 3],
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

            piece_bitboards: [0; 12],
            occupancy_bitboards: [0; 3],
        };
    }

    // Takes all pieces off the board.
    pub fn clear_board(&mut self) {
        self.piece_bitboards = [0; 12];
        self.occupancy_bitboards = [0; 3];
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

        // Piece bitboard.
        let piece_bitboard_index = piece_type.bitboard_index() + side.piece_bitboard_offset();
        self.piece_bitboards[piece_bitboard_index] = set_bit(self.piece_bitboards[piece_bitboard_index], square);

        // Color occupancies.
        let occupancy_bitboard_index = side.occupancy_bitboard_index();
        self.occupancy_bitboards[occupancy_bitboard_index] = set_bit(self.occupancy_bitboards[occupancy_bitboard_index], square);

        // All occupancies.
        self.occupancy_bitboards[2] = set_bit(self.occupancy_bitboards[2], square);
    }

    // WARNING: Not efficient function??
    pub fn get_piece_at_square(&self, square: usize) -> (Option<PieceType>, Option<Color>) {
        let is_occupied = get_bit(self.occupancy_bitboards[2], square) != 0;
        if !is_occupied {
            return (None, None);
        }

        let is_occupied_white = get_bit(self.occupancy_bitboards[0], square) != 0;
        let is_occupied_black = get_bit(self.occupancy_bitboards[1], square) != 0;

        let piece_color: Color;
        if is_occupied_white {
            piece_color = Color::White;
        } else if is_occupied_black {
            piece_color = Color::Black;
        } else {
            panic!("Someting has gone very wrong. All occupancies populated, but no piece found.");
        }

        let start_bitboard_index: usize = piece_color.piece_bitboard_offset();

        // Loop over all the piece bitboards until we find the piece we want.
        for i in start_bitboard_index..start_bitboard_index + 6 {
            if get_bit(self.piece_bitboards[i], square) != 0 {
                return (Some(PieceType::bitboard_index_to_piece_type(i)), Some(piece_color));
            }
        }

        panic!("Someting has gone very wrong. Looked at all bitboards and could not find a piece.");
    }

    // Bitwise operations make this pretty quick.
    pub fn is_square_attacked(&self, square: usize, who_is_attacking: &Color) -> bool {
        let piece_bitboard_offset = who_is_attacking.piece_bitboard_offset();
        let all_occupancies = self.occupancy_bitboards[2];
        let opponent_pawn_attacks_index = match who_is_attacking {
            Color::White => Color::Black.idx(),
            Color::Black => Color::White.idx(),
        };

        // Pawns.
        if self.bitboard_constants.pawn_attacks[opponent_pawn_attacks_index][square]
            & self.piece_bitboards[0 + piece_bitboard_offset] != 0
        {
            return true;
        }

        // Bishops.
        if self.get_bishop_attacks(square, all_occupancies) & self.piece_bitboards[1 + piece_bitboard_offset] != 0 {
            return true;
        }

        // Knights.
        if self.bitboard_constants.knight_attacks[square] & self.piece_bitboards[2 + piece_bitboard_offset] != 0 {
            return true;
        }

        // Rooks.
        if self.get_rook_attacks(square, all_occupancies) & self.piece_bitboards[3 + piece_bitboard_offset] != 0 {
            return true;
        }

        // Queens. (we could speed this up slightly... look here for optimization if needed.)
        if self.get_queen_attacks(square, all_occupancies) & self.piece_bitboards[4 + piece_bitboard_offset] != 0 {
            return true;
        }

        // King.
        if self.bitboard_constants.king_attacks[square] & self.piece_bitboards[5 + piece_bitboard_offset] != 0 {
            return true;
        }

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


    // pub fn make_move(&mut self, this_move: &Move, debugging: bool) {
    //     let (source_piece_unchecked, source_side_unchecked) = self.get_piece_at_square(this_move.from_square);
    //     let source_piece = match source_piece_unchecked {
    //         Some(p) => p,
    //         None => {
    //             panic!("Something has gone very wrong.");
    //         }
    //     };
    //     // Handle generic captures, and en-passant captures.
    //     let (target_piece,_) = self.get_piece_at_square(this_move.to_square);
    //     let our_color = source_side_unchecked.unwrap();
    //     let their_color = match our_color {
    //         Color::White => Color::Black,
    //         Color::Black => Color::White,
    //     };

    //     if debugging {
    //         println!("Debug output (start function) {:#?}", this_move);
    //         println!("Our Color: {:#?}\n Their Color: {:#?}", our_color, their_color);
    //     }

    //     // Remove our piece from it's starting square, and place it in the new spot.
    //     // This does not handle castling, and en-passant logic.
    //     match our_color {
    //         // Move the piece from it's source to the destination.
    //         Color::White => {
    //             match source_piece {
    //                 PieceType::Pawn => {
    //                     self.white_pawns = pop_bit(self.white_pawns, this_move.from_square);

    //                     if debugging {
    //                         println!("Looking at moving our pawns.");
    //                     }

    //                     // Special logic for pawn promotion.
    //                     match this_move.pawn_promoting_to {
    //                         Some(piece_promoted_to) => {
    //                             match piece_promoted_to {
    //                                 PieceType::Queen => self.white_queens = set_bit(self.white_queens, this_move.to_square),
    //                                 PieceType::Rook => self.white_rooks = set_bit(self.white_rooks, this_move.to_square),
    //                                 PieceType::Bishop => self.white_bishops = set_bit(self.white_bishops, this_move.to_square),
    //                                 PieceType::Knight => self.white_knights = set_bit(self.white_knights, this_move.to_square),
    //                                 _ => panic!("Tried to promote to an illegal piece."),
    //                             }
    //                         },
    //                         None => self.white_pawns = set_bit(self.white_pawns, this_move.to_square),
    //                     }
                        
    //                 },
    //                 PieceType::Bishop => {
    //                     self.white_bishops = pop_bit(self.white_bishops, this_move.from_square);
    //                     self.white_bishops = set_bit(self.white_bishops, this_move.to_square);
    //                 },
    //                 PieceType::Knight => {
    //                     self.white_knights = pop_bit(self.white_knights, this_move.from_square);
    //                     self.white_knights = set_bit(self.white_knights, this_move.to_square);
    //                 },
    //                 PieceType::Rook => {
    //                     self.white_rooks = pop_bit(self.white_rooks, this_move.from_square);
    //                     self.white_rooks = set_bit(self.white_rooks, this_move.to_square);
    //                 },
    //                 PieceType::Queen => {
    //                     self.white_queens = pop_bit(self.white_queens, this_move.from_square);
    //                     self.white_queens = set_bit(self.white_queens, this_move.to_square);
    //                 },
    //                 PieceType::King => {
    //                     self.white_king = pop_bit(self.white_king, this_move.from_square);
    //                     self.white_king = set_bit(self.white_king, this_move.to_square);
    //                 },
    //             }

    //             // Update our occupancies.
    //             self.white_occupancies = pop_bit(self.white_occupancies, this_move.from_square);
    //             self.white_occupancies = set_bit(self.white_occupancies, this_move.to_square);
    //         },
    //         Color::Black => {
    //             match source_piece {
    //                 PieceType::Pawn => {
    //                     self.black_pawns = pop_bit(self.black_pawns, this_move.from_square);
    //                     self.black_pawns = set_bit(self.black_pawns, this_move.to_square);
    //                 },
    //                 PieceType::Bishop => {
    //                     self.black_bishops = pop_bit(self.black_bishops, this_move.from_square);
    //                     self.black_bishops = set_bit(self.black_bishops, this_move.to_square);
    //                 },
    //                 PieceType::Knight => {
    //                     self.black_knights = pop_bit(self.black_knights, this_move.from_square);
    //                     self.black_knights = set_bit(self.black_knights, this_move.to_square);
    //                 },
    //                 PieceType::Rook => {
    //                     self.black_rooks = pop_bit(self.black_rooks, this_move.from_square);
    //                     self.black_rooks = set_bit(self.black_rooks, this_move.to_square);
    //                 },
    //                 PieceType::Queen => {
    //                     self.black_queens = pop_bit(self.black_queens, this_move.from_square);
    //                     self.black_queens = set_bit(self.black_queens, this_move.to_square);
    //                 },
    //                 PieceType::King => {
    //                     self.black_king = pop_bit(self.black_king, this_move.from_square);
    //                     self.black_king = set_bit(self.black_king, this_move.to_square);
    //                 },
    //             }

    //             // Update our occupancies.
    //             self.black_occupancies = pop_bit(self.black_occupancies, this_move.from_square);
    //             self.black_occupancies = set_bit(self.black_occupancies, this_move.to_square);
    //         }
    //     }

    //     // Update all occupancies, source piece always moves.
    //     self.all_occupancies = pop_bit(self.all_occupancies, this_move.from_square);

    //     // Figure out if we are capturing.
    //     if this_move.to_piece_type.is_none() {
    //         panic!("Tried to make a move, but not sure if it was a capture or not. Cannot proceed.");
    //     }

    //     let is_capture = this_move.to_piece_type.is_some();
    //     if !is_capture {
    //         if debugging {
    //             println!("This move is not a capture. Add occupancy to destination.");
    //         }
    //         self.all_occupancies = set_bit(self.all_occupancies, this_move.to_square);
    //     } else {
    //         match target_piece {
    //             Some(piece_on_target_square) => {
    //                 match their_color {
    //                     // Move the piece from it's source to the destination.
    //                     Color::White => {
    //                         match piece_on_target_square {
    //                             PieceType::Pawn => {
    //                                 self.white_pawns = pop_bit(self.white_pawns, this_move.to_square);
    //                             },
    //                             PieceType::Bishop => {
    //                                 self.white_bishops = pop_bit(self.white_bishops, this_move.to_square);
    //                             },
    //                             PieceType::Knight => {
    //                                 self.white_knights = pop_bit(self.white_knights, this_move.to_square);
    //                             },
    //                             PieceType::Rook => {
    //                                 self.white_rooks = pop_bit(self.white_rooks, this_move.to_square);
    //                             },
    //                             PieceType::Queen => {
    //                                 self.white_queens = pop_bit(self.white_queens, this_move.to_square);
    //                             },
    //                             PieceType::King => {
    //                                 panic!("You cannot capture the king.");
    //                             },
    //                         }

    //                         // Update their occupancies.
    //                         self.white_occupancies = pop_bit(self.white_occupancies, this_move.to_square);
    //                     },
    //                     Color::Black => {
    //                         if debugging {
    //                             println!("Handling capture. Their color is black. Piece on their square is: {:#?}", piece_on_target_square);
    //                         }
    //                         match piece_on_target_square {
    //                             PieceType::Pawn => {
    //                                 self.black_pawns = pop_bit(self.black_pawns, this_move.to_square);
    //                             },
    //                             PieceType::Bishop => {
    //                                 self.black_bishops = pop_bit(self.black_bishops, this_move.to_square);
    //                             },
    //                             PieceType::Knight => {
    //                                 self.black_knights = pop_bit(self.black_knights, this_move.to_square);
    //                             },
    //                             PieceType::Rook => {
    //                                 if debugging {
    //                                     println!("Before popping black rook.");
    //                                     print_bitboard(self.black_rooks);
    //                                 }
                                    
    //                                 self.black_rooks = pop_bit(self.black_rooks, this_move.to_square);

    //                                 if debugging {
    //                                     println!("After popping black rook.");
    //                                     print_bitboard(self.black_rooks);
    //                                 }
    //                             },
    //                             PieceType::Queen => {
    //                                 self.black_queens = pop_bit(self.black_queens, this_move.to_square);
    //                             },
    //                             PieceType::King => {
    //                                 panic!("You cannot capture the king.");
    //                             },
    //                         }

    //                         // Update their occupancies.
    //                         self.black_occupancies = pop_bit(self.black_occupancies, this_move.to_square);
    //                     }
    //                 }
    //             },

    //             // This is an en-passant capture. Treat it as such.
    //             None => {
    //                 if debugging {
    //                     println!("Handling en-passant capture...");
    //                 }
    //                 self.all_occupancies = set_bit(self.all_occupancies, this_move.to_square);
    //                 match their_color {
    //                     Color::White => {
    //                         self.white_pawns = pop_bit(self.white_pawns, this_move.to_square - 8);
    //                         self.white_occupancies = pop_bit(self.white_occupancies, this_move.to_square - 8);
    //                         self.all_occupancies = pop_bit(self.all_occupancies, this_move.to_square - 8);
    //                     },
    //                     Color::Black => {
    //                         self.black_pawns = pop_bit(self.black_pawns, this_move.to_square + 8);
    //                         self.black_occupancies = pop_bit(self.black_occupancies, this_move.to_square + 8);
    //                         self.all_occupancies = pop_bit(self.all_occupancies, this_move.to_square + 8);
    //                     },
    //                 }
    //             }
    //         }
    //     }

    //     // Lastly, handle castling.
    //     match this_move.castle_side {
    //         None => (),
    //         Some(side) => {
    //             let king_from_position = this_move.from_square;
    //             let rook_from_position = match side {
    //                 CastleSides::Short => king_from_position + 3,
    //                 CastleSides::Long => king_from_position - 4,
    //             };
    //             let rook_to_position = match side {
    //                 CastleSides::Short => king_from_position + 1,
    //                 CastleSides::Long => king_from_position - 1,
    //             };
    //             match our_color {
    //                 Color::White => {
    //                     self.white_rooks = pop_bit(self.white_rooks, rook_from_position);
    //                     self.white_rooks = set_bit(self.white_rooks, rook_to_position);
    //                     self.white_occupancies = pop_bit(self.white_occupancies, rook_from_position);
    //                     self.white_occupancies = set_bit(self.white_occupancies, rook_to_position);
    //                 },
    //                 Color::Black => {
    //                     self.black_rooks = pop_bit(self.black_rooks, rook_from_position);
    //                     self.black_rooks = set_bit(self.black_rooks, rook_to_position);
    //                     self.black_occupancies = pop_bit(self.black_occupancies, rook_from_position);
    //                     self.black_occupancies = set_bit(self.black_occupancies, rook_to_position);
    //                 }
    //             }

    //             self.all_occupancies = pop_bit(self.all_occupancies, rook_from_position);
    //             self.all_occupancies = set_bit(self.all_occupancies, rook_to_position);
    //         }
    //     }


    //     // Forfeiting castling rights.
    //     if source_piece == PieceType::King {
    //         match our_color {
    //             Color::White => {
    //                 self.can_white_castle_short = false;
    //                 self.can_white_castle_long = false;
    //             },
    //             Color::Black => {
    //                 self.can_black_castle_short = false;
    //                 self.can_black_castle_long = false;
    //             },
    //         }
    //     }

    //     if source_piece == PieceType::Rook {
    //         match our_color {
    //             Color::White => {
    //                 if this_move.from_square == 63 {
    //                     self.can_white_castle_short = false;
    //                 } else if this_move.from_square == 56 {
    //                     self.can_white_castle_long = false;
    //                 }
    //             },
    //             Color::Black => {
    //                 if this_move.from_square == 7 {
    //                     self.can_black_castle_short = false;
    //                 } else if this_move.from_square == 0 {
    //                     self.can_black_castle_long = false;
    //                 }
    //             },
    //         }
    //     }

    //     // Only needed if we are capturing.
    //     self.en_passant_target = this_move.next_en_passant_target_coord;

    //     // Important for checking if move is illegal.
    //     self.white_to_move = !self.white_to_move;
    // }


    pub fn is_king_attacked(&self, side_attacked: &Color) -> bool {
        let king_bitboard_index = 5 + side_attacked.piece_bitboard_offset();
        let king_square = get_lsb_index(self.piece_bitboards[king_bitboard_index]).expect("King must be on board.");
        return match side_attacked {
            Color::White => self.is_square_attacked(king_square, &Color::Black),
            Color::Black => self.is_square_attacked(king_square, &Color::White),
        };
    }


    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let possible_moves = self.get_psuedo_legal_moves();
        let our_side;

        if self.white_to_move {
            our_side = &Color::White;
        } else {
            our_side = &Color::Black;
        }

        // moves.push(*this_move);

        // Try the move, drop it if it's illegal.
        // for this_move in possible_moves.iter() {
        //     let mut self_copy = self.clone();
        //     self_copy.make_move(this_move, false);
        //     if !self_copy.is_king_attacked(our_side) {
        //         moves.push(*this_move);
        //     }
        // }

        // return moves;
        return possible_moves;
    }

    // Will generate moves that put self in check.
    pub fn get_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> =  vec![];

        // Get all the moves.
        moves.append(&mut self.get_moves_slider(PieceType::Queen));
        moves.append(&mut self.get_moves_slider(PieceType::Rook));
        moves.append(&mut self.get_moves_slider(PieceType::Bishop));
        moves.append(&mut self.get_moves_knight());
        // moves.append(&mut self.get_moves_king());
        // moves.append(&mut self.get_moves_pawns());

        return moves;
    }

    pub fn print_legal_moves(&self) {
        let moves = self.get_legal_moves();
        for m in moves.iter() {
            print!("{} ", m.move_to_str());
        }
        if moves.len() == 0 {
            println!("There are no legal moves...");
        }
        print!("\n");
    }

    pub fn get_moves_slider(&self, slider_piece_type: PieceType) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let mut source_square: usize;
        let mut target_square: usize;
        let mut slider_pieces: u64;
        let mut slider_piece_attacks: u64;
        let mut quiet_moves: u64;
        let mut captures: u64;
        let mut removes_castling_rights_short: bool;
        let mut removes_castling_rights_long: bool;
        let mut to_piece_type: Option<PieceType>;

        let all_occupancies: u64 = self.occupancy_bitboards[2];
        let their_occupancies: u64;
        let our_piece_bitboard_offset: usize;
        let rook_square_offset: usize;

        if self.white_to_move {
            their_occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
            our_piece_bitboard_offset = Color::White.piece_bitboard_offset();
            rook_square_offset = 56;
        } else {
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            our_piece_bitboard_offset = Color::Black.piece_bitboard_offset();
            rook_square_offset = 0;
        }

        slider_pieces = match slider_piece_type {
            PieceType::Queen => {
                self.piece_bitboards[our_piece_bitboard_offset + PieceType::Queen.bitboard_index()]
            }
            PieceType::Rook => {
                self.piece_bitboards[our_piece_bitboard_offset + PieceType::Rook.bitboard_index()]
            }
            PieceType::Bishop => {
                self.piece_bitboards[our_piece_bitboard_offset + PieceType::Bishop.bitboard_index()]
            }
            _ => {
                panic!("Attempted to get slider piece moves for non-slider piece.");
            }
        };

        // println!("Our slider pieces bitboard:");
        // print_bitboard(slider_pieces);

        while slider_pieces != 0 {
            source_square = get_lsb_index(slider_pieces).expect("This should not happen.");
            removes_castling_rights_short = false;
            removes_castling_rights_long = false;

            // Get moves and captures seperately.
            match slider_piece_type {
                PieceType::Queen => {
                    slider_piece_attacks = self.get_queen_attacks(source_square, all_occupancies);
                }
                PieceType::Rook => {
                    slider_piece_attacks = self.get_rook_attacks(source_square, all_occupancies);

                    // We need to check if this move would remove castling rights.
                    if source_square == rook_square_offset && self.white_to_move && self.can_white_castle_long {
                        removes_castling_rights_long = true;
                    } else if source_square == rook_square_offset + 7 && self.white_to_move && self.can_white_castle_short {
                        removes_castling_rights_short = true;
                    } else if source_square == rook_square_offset && !self.white_to_move && self.can_black_castle_long {
                        removes_castling_rights_long = true;
                    } else if source_square == rook_square_offset + 7 && !self.white_to_move && self.can_black_castle_short {
                        removes_castling_rights_short = true;
                    }
                }
                PieceType::Bishop => {
                    slider_piece_attacks = self.get_bishop_attacks(source_square, all_occupancies);
                }
                _ => {
                    panic!("Tried to get slider piece attacks for non-slider piece.");
                }
            };

            quiet_moves = slider_piece_attacks & (!all_occupancies);
            captures = slider_piece_attacks & their_occupancies;

            while quiet_moves != 0 {
                target_square = get_lsb_index(quiet_moves).expect("This should not be empty.");
                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(slider_piece_type),
                    to_square: target_square,
                    to_piece_type: None,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    is_check: None,
                    pawn_promoting_to: None,
                    removes_castling_rights_short: removes_castling_rights_short,
                    removes_castling_rights_long: removes_castling_rights_long,
                    castle_side: None,
                });
                quiet_moves = pop_bit(quiet_moves, target_square);
            }

            while captures != 0 {
                target_square = get_lsb_index(captures).expect("This should not be empty.");
                (to_piece_type, _) = self.get_piece_at_square(target_square);

                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(slider_piece_type),
                    to_square: target_square,
                    to_piece_type: to_piece_type,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    is_check: None,
                    pawn_promoting_to: None,
                    removes_castling_rights_short: removes_castling_rights_short,
                    removes_castling_rights_long: removes_castling_rights_long,
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
        let mut to_piece_type: Option<PieceType>;
        let their_occupancies: u64;

        if self.white_to_move {
            their_occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
            knights = self.piece_bitboards[Color::White.piece_bitboard_offset() + PieceType::Knight.bitboard_index()];
        } else {
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            knights = self.piece_bitboards[Color::Black.piece_bitboard_offset() + PieceType::Knight.bitboard_index()];
        }

        while knights != 0 {
            source_square = get_lsb_index(knights).expect("This should not happen.");

            // Get moves and captures seperately.
            quiet_moves = self.bitboard_constants.knight_attacks[source_square] & (!self.occupancy_bitboards[2]);
            captures = self.bitboard_constants.knight_attacks[source_square] & their_occupancies;

            while quiet_moves != 0 {
                target_square = get_lsb_index(quiet_moves).expect("This should not be empty.");
                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(PieceType::Knight),
                    to_square: target_square,
                    to_piece_type: None,
                    is_check: None,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                    removes_castling_rights_short: false,
                    removes_castling_rights_long: false,
                });
                quiet_moves = pop_bit(quiet_moves, target_square);
            }

            while captures != 0 {
                target_square = get_lsb_index(captures).expect("This should not be empty.");
                (to_piece_type, _) = self.get_piece_at_square(target_square);

                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(PieceType::Knight),
                    to_square: target_square,
                    to_piece_type: to_piece_type,
                    is_check: None,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    pawn_promoting_to: None,
                    castle_side: None,
                    removes_castling_rights_short: false,
                    removes_castling_rights_long: false,
                });
                captures = pop_bit(captures, target_square);
            }

            knights = pop_bit(knights, source_square);
        }

        return moves;
    }

    // pub fn get_moves_king(&self) -> Vec<Move> {
    //     let mut moves: Vec<Move> = vec![];
    //     let source_square: usize;
    //     let mut target_square: usize;
    //     let bitboard: u64;

    //     let their_color: &Color;
    //     let their_occupancies: u64;
    //     let can_castle_long: bool;
    //     let can_castle_short: bool;
    //     let king_starting_square: usize;
    //     if self.white_to_move {
    //         their_color = &Color::Black;
    //         their_occupancies = self.black_occupancies;
    //         bitboard = self.white_king;
    //         can_castle_short = self.can_white_castle_short;
    //         can_castle_long = self.can_white_castle_long;
    //         king_starting_square = 60;
    //     } else {
    //         their_color = &Color::White;
    //         their_occupancies = self.white_occupancies;
    //         bitboard = self.black_king;
    //         can_castle_short = self.can_black_castle_short;
    //         can_castle_long = self.can_black_castle_long;
    //         king_starting_square = 4;
    //     }

    //     if bitboard == 0 {
    //         return moves;
    //     }

    //     source_square = get_lsb_index(bitboard).expect("Guard before should handle this.");
    //     let mut quiet_moves = self.bitboard_constants.king_attacks[source_square] & (!self.all_occupancies);
    //     let mut attacks = self.bitboard_constants.king_attacks[source_square] & their_occupancies;


    //     // Moves
    //     while quiet_moves != 0 {
    //         target_square = get_lsb_index(quiet_moves).expect("Guard before should handle this.");
    //         moves.push(Move {
    //             from_square: source_square,
    //             to_square: target_square,
    //             is_check: None,
    //             next_en_passant_target_coord: None,
    //             pawn_promoting_to: None,
    //             castle_side: None,
    //         });
    //         quiet_moves = pop_bit(quiet_moves, target_square);
    //     }

    //     // Attacks
    //     while attacks != 0 {
    //         target_square = get_lsb_index(attacks).expect("Guard before should handle this.");
    //         moves.push(Move {
    //             from_square: source_square,
    //             to_square: target_square,
    //             is_check: None,
    //             next_en_passant_target_coord: None,
    //             pawn_promoting_to: None,
    //             castle_side: None,
    //         });
    //         attacks = pop_bit(attacks, target_square);
    //     }

    //     // Castling
    //     if can_castle_short {

    //         // 1. Make sure squares are empty.
    //         let squares_should_be_empty = set_bit(0, king_starting_square + 1) | set_bit(0, king_starting_square + 2);

    //         // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
    //         let is_intermediary_square_attacked = self.is_square_attacked(king_starting_square + 1, their_color);

    //         // If both conditions are met, we can castle.
    //         target_square = king_starting_square + 2;
    //         if (squares_should_be_empty & self.all_occupancies) == 0 && !is_intermediary_square_attacked {
    //             moves.push(Move {
    //                 from_square: source_square,
    //                 to_square: target_square,
    //                 is_check: None,
    //                 next_en_passant_target_coord: None,
    //                 pawn_promoting_to: None,
    //                 castle_side: Some(CastleSides::Short),
    //             });
    //         }
    //     }

    //     if can_castle_long {

    //         // 1. Make sure squares are empty.
    //         let squares_should_be_empty = set_bit(0, king_starting_square - 1) | set_bit(0, king_starting_square - 2) | set_bit(0, king_starting_square - 3);

    //         // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
    //         let is_intermediary_square_attacked = self.is_square_attacked(king_starting_square - 1, their_color);

    //         // If both conditions are met, we can castle.
    //         target_square = king_starting_square - 2;
    //         if (squares_should_be_empty & self.all_occupancies) == 0 && !is_intermediary_square_attacked {
    //             moves.push(Move {
    //                 from_square: source_square,
    //                 to_square: target_square,
    //                 is_check: None,
    //                 next_en_passant_target_coord: None,
    //                 pawn_promoting_to: None,
    //                 castle_side: Some(CastleSides::Long),
    //             });
    //         }
    //     }

    //     return moves;
    // }

    // pub fn get_moves_pawns(&self) -> Vec<Move> {
    //     let mut moves: Vec<Move> = vec![];
    //     let mut source_square: usize;
    //     let mut target_square: usize;

    //     let mut bitboard: u64;
    //     let mut attacks: u64;

    //     let our_color: Color;
    //     let their_occupancies: u64;
    //     let pawn_move_offset: i32;
    //     let promotion_rank_lower: usize;
    //     let promotion_rank_upper: usize;
    //     let our_starting_rank_lower: usize;
    //     let our_starting_rank_upper: usize;
    //     if self.white_to_move {
    //         our_color = Color::White;
    //         their_occupancies = self.black_occupancies;
    //         bitboard = self.white_pawns;
    //         pawn_move_offset = -8;
    //         promotion_rank_lower = 0;
    //         promotion_rank_upper = 7;
    //         our_starting_rank_lower = 48;
    //         our_starting_rank_upper = 55;
    //     } else {
    //         our_color = Color::Black;
    //         their_occupancies = self.white_occupancies;
    //         bitboard = self.black_pawns;
    //         pawn_move_offset = 8;
    //         promotion_rank_lower = 56;
    //         promotion_rank_upper = 63;
    //         our_starting_rank_lower = 8;
    //         our_starting_rank_upper = 15;
    //     }

    //     while bitboard != 0 {
    //         source_square = get_lsb_index(bitboard).expect("This should not fail.");

    //         // Handles forward moves.
    //         target_square = (source_square as i32 + pawn_move_offset) as usize;
    //         let mut is_occupied = get_bit(self.all_occupancies, target_square) != 0;
    //         if !is_occupied {

    //             // Check for promotions (no capture).
    //             if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Queen),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Rook),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Bishop),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Knight),
    //                     castle_side: None,
    //                 });
                    
    //             } else {
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: None,
    //                     castle_side: None,
    //                 });

    //                 // Check for the double move.
    //                 target_square = (source_square as i32 + pawn_move_offset + pawn_move_offset) as usize;
    //                 is_occupied = get_bit(self.all_occupancies, target_square) != 0;

    //                 // If pawn is on the 2nd rank, it can move two tiles.
    //                 if source_square >= our_starting_rank_lower && source_square <= our_starting_rank_upper && !is_occupied {
    //                     moves.push(Move {
    //                         from_square: source_square,
    //                         to_square: target_square,
    //                         is_check: None,
    //                         next_en_passant_target_coord: Some((source_square as i32 + pawn_move_offset) as usize),
    //                         pawn_promoting_to: None,
    //                         castle_side: None,
    //                     });
    //                 }
    //             }
    //         }

    //         // Handles captures (non-en-passant).
    //         attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square] & their_occupancies;
    //         while attacks != 0 {
    //             target_square = get_lsb_index(attacks).expect("Should not be empty.");
    //             if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Queen),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Rook),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Bishop),
    //                     castle_side: None,
    //                 });
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: Some(PieceType::Knight),
    //                     castle_side: None,
    //                 });
    //             } else {
    //                 moves.push(Move {
    //                     from_square: source_square,
    //                     to_square: target_square,
    //                     is_check: None,
    //                     next_en_passant_target_coord: None,
    //                     pawn_promoting_to: None,
    //                     castle_side: None,
    //                 });
    //             }
    //             attacks = pop_bit(attacks, target_square);
    //         }

    //         // Handles captures (en-passant)
    //         match self.en_passant_target {
    //             Some(s) => {
    //                 attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square] & set_bit(0, s);
    //                 if attacks != 0 {
    //                     moves.push(Move {
    //                         from_square: source_square,
    //                         to_square: target_square,
    //                         is_check: None,
    //                         next_en_passant_target_coord: None,
    //                         pawn_promoting_to: None,
    //                         castle_side: None,
    //                     });
    //                 }
    //             }
    //             _ => ()
    //         }


    //         // Empty the board! and go next.
    //         bitboard = pop_bit(bitboard, source_square);
    //     }

    //     return moves;
    // }
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
