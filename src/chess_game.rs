use crate::castle_sides::CastleSides;
use crate::color::Color;
use crate::constants;
use crate::helpers::*;
use crate::piece_type::PieceType;
use crate::r#move::Move;
use crate::runtime_calculated_constants::Constants;
use crate::transposition_table_entry::{TranspositionTableEntry, TranspositionTableNodeType};
use std::collections::HashMap;
use std::io;

// TODO: Research more on lifetime stuff.
#[derive(Clone)]
pub struct ChessGame<'a> {
    pub bitboard_constants: &'a Constants,

    pub zobrist_hash: u64,

    pub transposition_table: HashMap<u64, TranspositionTableEntry>,

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

    pub legal_moves: Vec<Move>,

    pub debug_minimax_calls: u64,
    pub debug_mimimax_moves_made: Vec<Move>,
}

impl<'a> ChessGame<'a> {
    pub fn new(c: &'a Constants) -> Self {
        return ChessGame {
            bitboard_constants: c,

            zobrist_hash: 0,
            transposition_table: HashMap::new(),

            en_passant_target: None,

            white_to_move: true,
            can_white_castle_long: true,
            can_white_castle_short: true,
            can_black_castle_long: true,
            can_black_castle_short: true,

            piece_bitboards: [0; 12],
            occupancy_bitboards: [0; 3],

            legal_moves: vec![],

            debug_minimax_calls: 0,
            debug_mimimax_moves_made: vec![],
        };
    }

    pub fn debug_verify_board_state(
        &self,
        this_move: &Move,
        prev_game_state: ChessGame,
        called_by: &str,
    ) {
        // Make sure occupancies add up corrrectly.
        if self.occupancy_bitboards[2] != self.occupancy_bitboards[0] | self.occupancy_bitboards[1]
        {
            println!("Previous game state");
            prev_game_state.print_board();
            println!(
                "Tried to make/unmake move: {:#?}\n\n Ended up with:",
                this_move
            );

            println!("White occupancies before:");
            print_bitboard(prev_game_state.occupancy_bitboards[0]);
            println!("White occupancies after:");
            print_bitboard(self.occupancy_bitboards[0]);

            println!("Black occupancies before:");
            print_bitboard(prev_game_state.occupancy_bitboards[1]);
            println!("Black occupancies after:");
            print_bitboard(self.occupancy_bitboards[1]);

            println!("All occupancies before:");
            print_bitboard(prev_game_state.occupancy_bitboards[2]);
            println!("All occupancies after:");
            print_bitboard(self.occupancy_bitboards[2]);

            panic!("Occupancy desynced by {called_by}.");
        }

        // Make sure white pieces add up to occupancies.
        let mut constructed_occupancies_white = 0;
        for i in 0..6 {
            constructed_occupancies_white |= self.piece_bitboards[i];
        }
        if self.occupancy_bitboards[0] != constructed_occupancies_white {
            println!("Previous game state");
            prev_game_state.print_board();
            println!(
                "Tried to make/unmake move: {:#?}\n\n Ended up with:",
                this_move
            );

            println!("White rooks before:");
            print_bitboard(prev_game_state.piece_bitboards[PieceType::Rook.bitboard_index()]);
            println!("White rooks after:");
            print_bitboard(self.piece_bitboards[PieceType::Rook.bitboard_index()]);

            println!("Moves that lead to desyc:");
            for m in self.debug_mimimax_moves_made.iter() {
                print!("{}, ", m.move_to_str());
            }
            println!();

            panic!("White pieces desynced by {called_by}.");
        }

        let mut constructed_occupancies_black = 0;
        for i in 6..12 {
            constructed_occupancies_black |= self.piece_bitboards[i];
        }
        if self.occupancy_bitboards[1] != constructed_occupancies_black {
            println!("Previous game state");
            prev_game_state.print_board();
            println!(
                "Tried to make/unmake move: {:#?}\n\n Ended up with:",
                this_move
            );

            panic!("Black pieces desynced by {called_by}.");
        }
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

        // Reset the zobrist hash.
        self.zobrist_hash = 0;

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

                // Update the zobrist hash.
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [piece_type.bitboard_index() + piece_color.piece_bitboard_offset()][square];

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
            // Zobrist hash assumes white to move initially.
        } else if whose_turn.to_ascii_lowercase() == "b" {
            self.white_to_move = false;

            // XOR if it's black to move.
            self.zobrist_hash ^= self.bitboard_constants.zobrist_to_move;
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
                        'K' => {
                            self.can_white_castle_short = true;
                            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[0];
                        }
                        'Q' => {
                            self.can_white_castle_long = true;
                            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[1];
                        }
                        'k' => {
                            self.can_black_castle_short = true;
                            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[2];
                        }
                        'q' => {
                            self.can_black_castle_long = true;
                            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[3];
                        }
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
                    let square = parsed_coord.unwrap();
                    self.en_passant_target = Some(square);
                    self.zobrist_hash ^= self.bitboard_constants.zobrist_en_passant[square % 8];
                } else {
                    self.en_passant_target = None;
                }
            }
            None => return Ok(()),
        };

        return Ok(());
    }

    pub fn export_fen(&self) -> String {
        let mut fen = String::new();
        let mut prior_empty_count = 0;
        let mut file;
        for square in 0..64 {
            // Track our file.
            file = square % 8;

            // Get the piece at this square.
            let (piece_option, color_option) = self.get_piece_at_square(square);

            match piece_option {
                Some(piece) => {
                    // If we had spaces before, print that and reset.
                    if prior_empty_count > 0 {
                        fen += &prior_empty_count.to_string();
                        prior_empty_count = 0;
                    }

                    let color = color_option.expect("Piece returned without a color?");
                    fen += &piece.to_char(color).to_string();
                }
                None => {
                    prior_empty_count += 1;
                }
            }

            // If we are on the last file
            if file == 7 {
                // If we have any empties, output the number.
                if prior_empty_count > 0 {
                    fen += &prior_empty_count.to_string();
                }

                // Clear the empty count.
                prior_empty_count = 0;

                // Add the slash, if not on the last row.
                if square != 63 {
                    fen += "/";
                }
            }
        }

        // Add the game status (turn to move)
        if self.white_to_move {
            fen += " w ";
        } else {
            fen += " b ";
        }

        // Castling rights.
        if self.can_white_castle_short {
            fen += "K";
        }
        if self.can_white_castle_long {
            fen += "Q";
        }
        if self.can_black_castle_short {
            fen += "k";
        }
        if self.can_black_castle_long {
            fen += "q";
        }

        // Special case if no casting rights available.
        if !self.can_white_castle_short
            && !self.can_white_castle_long
            && !self.can_black_castle_short
            && !self.can_black_castle_long
        {
            fen += "-";
        }

        // En-Passant Square.
        fen += " ";
        match self.en_passant_target {
            Some(square) => fen += &square_to_coord(square),
            None => fen += "-",
        }

        // Ehhh, at some point add the half moves since last capture or pawn advance. And the full move count.

        return fen;
    }

    pub fn place_piece_on_board(&mut self, side: Color, piece_type: PieceType, square: usize) {
        // Piece bitboard.
        let piece_bitboard_index = piece_type.bitboard_index() + side.piece_bitboard_offset();
        self.piece_bitboards[piece_bitboard_index] =
            set_bit(self.piece_bitboards[piece_bitboard_index], square);

        // Color occupancies.
        let occupancy_bitboard_index = side.occupancy_bitboard_index();
        self.occupancy_bitboards[occupancy_bitboard_index] =
            set_bit(self.occupancy_bitboards[occupancy_bitboard_index], square);

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
                return (
                    Some(PieceType::bitboard_index_to_piece_type(i)),
                    Some(piece_color),
                );
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
            & self.piece_bitboards[0 + piece_bitboard_offset]
            != 0
        {
            return true;
        }

        // Bishops.
        if self.get_bishop_attacks(square, all_occupancies)
            & self.piece_bitboards[1 + piece_bitboard_offset]
            != 0
        {
            return true;
        }

        // Knights.
        if self.bitboard_constants.knight_attacks[square]
            & self.piece_bitboards[2 + piece_bitboard_offset]
            != 0
        {
            return true;
        }

        // Rooks.
        if self.get_rook_attacks(square, all_occupancies)
            & self.piece_bitboards[3 + piece_bitboard_offset]
            != 0
        {
            return true;
        }

        // Queens. (we could speed this up slightly... look here for optimization if needed.)
        if self.get_queen_attacks(square, all_occupancies)
            & self.piece_bitboards[4 + piece_bitboard_offset]
            != 0
        {
            return true;
        }

        // King.
        if self.bitboard_constants.king_attacks[square]
            & self.piece_bitboards[5 + piece_bitboard_offset]
            != 0
        {
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

    pub fn make_move(&mut self, this_move: &Move, update_legal_moves: bool) {
        //let debug_initial_game_state = self.clone();

        let source_piece = this_move
            .from_piece_type
            .expect("This should always be here.");

        // Handle generic captures, and en-passant captures.
        let our_color: Color;
        let their_color: Color;

        if self.white_to_move {
            our_color = Color::White;
            their_color = Color::Black;
        } else {
            our_color = Color::Black;
            their_color = Color::White;
        }

        let our_piece_bitboard_offset: usize = our_color.piece_bitboard_offset();
        let our_piece_bitboard_index: usize =
            our_piece_bitboard_offset + source_piece.bitboard_index();
        let our_occupancies_index: usize = our_color.occupancy_bitboard_index();
        let their_piece_bitboard_offset: usize = their_color.piece_bitboard_offset();
        let their_occupancies_index: usize = their_color.occupancy_bitboard_index();

        // Remove our piece from it's starting square, and place it in the new spot.
        // This does not handle castling, and en-passant logic.
        match source_piece {
            PieceType::Pawn => {
                self.piece_bitboards[our_piece_bitboard_index] = pop_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.from_square,
                );
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.from_square];

                // Special logic for pawn promotion.
                match this_move.pawn_promoting_to {
                    Some(piece_promoted_to) => {
                        match piece_promoted_to {
                            PieceType::Queen => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = set_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Rook => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = set_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Bishop => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = set_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Knight => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = set_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            _ => panic!("Tried to promote to an illegal piece."),
                        }
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [our_piece_bitboard_offset + piece_promoted_to.bitboard_index()]
                            [this_move.to_square];
                    }

                    // Otherwise, it's a normal pawn move.
                    None => {
                        self.piece_bitboards[our_piece_bitboard_index] = set_bit(
                            self.piece_bitboards[our_piece_bitboard_index],
                            this_move.to_square,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [our_piece_bitboard_index][this_move.to_square];
                    }
                }
            }

            // Every other piece, remove it from the source, place it at the destination.
            _ => {
                self.piece_bitboards[our_piece_bitboard_index] = pop_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.from_square,
                );
                self.piece_bitboards[our_piece_bitboard_index] = set_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.to_square,
                );

                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.from_square];
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.to_square];
            }
        }

        // Update our occupancies.
        self.occupancy_bitboards[our_occupancies_index] = pop_bit(
            self.occupancy_bitboards[our_occupancies_index],
            this_move.from_square,
        );
        self.occupancy_bitboards[our_occupancies_index] = set_bit(
            self.occupancy_bitboards[our_occupancies_index],
            this_move.to_square,
        );

        // Update all occupancies, source piece always moves.
        self.occupancy_bitboards[2] = pop_bit(self.occupancy_bitboards[2], this_move.from_square);

        // Figure out if we are capturing.
        let is_capture = this_move.to_piece_type.is_some();
        if !is_capture {
            self.occupancy_bitboards[2] = set_bit(self.occupancy_bitboards[2], this_move.to_square);
        } else {
            let their_piece = this_move
                .to_piece_type
                .expect("Should be here, thanks to guard.");
            let their_piece_bitboard_index =
                their_piece.bitboard_index() + their_piece_bitboard_offset;

            // Remove their piece from the square; and update their occupancies.
            match their_piece {
                // For pawn captures, handle en-passant.
                PieceType::Pawn => {
                    if this_move.is_en_passant_capture {
                        // Place our pawn on the target square. Normal captures do not need to update this, but en-passant does.
                        self.occupancy_bitboards[2] =
                            set_bit(self.occupancy_bitboards[2], this_move.to_square);
                        let en_passant_target_pawn_index: usize = match their_color {
                            Color::White => this_move.to_square - 8,
                            Color::Black => this_move.to_square + 8,
                        };

                        // Remove their pawn we captured en-passant.
                        self.piece_bitboards
                            [their_piece_bitboard_offset + their_piece.bitboard_index()] = pop_bit(
                            self.piece_bitboards
                                [their_piece_bitboard_offset + their_piece.bitboard_index()],
                            en_passant_target_pawn_index,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [their_piece_bitboard_offset + their_piece.bitboard_index()]
                            [en_passant_target_pawn_index];

                        // Remove their occupancy.
                        self.occupancy_bitboards[their_occupancies_index] = pop_bit(
                            self.occupancy_bitboards[their_occupancies_index],
                            en_passant_target_pawn_index,
                        );

                        // Remove global occupancy.
                        self.occupancy_bitboards[2] =
                            pop_bit(self.occupancy_bitboards[2], en_passant_target_pawn_index);
                    } else {
                        // Remove that piece from the board.
                        self.piece_bitboards[their_piece_bitboard_index] = pop_bit(
                            self.piece_bitboards[their_piece_bitboard_index],
                            this_move.to_square,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [their_piece_bitboard_index][this_move.to_square];

                        // Update their occupancies.
                        self.occupancy_bitboards[their_occupancies_index] = pop_bit(
                            self.occupancy_bitboards[their_occupancies_index],
                            this_move.to_square,
                        );
                    }
                }

                // For all non-pawn captures...
                _ => {
                    // Remove that piece from the board.
                    self.piece_bitboards[their_piece_bitboard_index] = pop_bit(
                        self.piece_bitboards[their_piece_bitboard_index],
                        this_move.to_square,
                    );
                    self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                        [their_piece_bitboard_index][this_move.to_square];

                    // Update their occupancies.
                    self.occupancy_bitboards[their_occupancies_index] = pop_bit(
                        self.occupancy_bitboards[their_occupancies_index],
                        this_move.to_square,
                    );
                }
            }
        }

        // Lastly, handle castling.
        match this_move.castle_side {
            None => (),
            Some(side) => {
                let king_from_position = this_move.from_square;
                let rook_from_position = match side {
                    CastleSides::Short => king_from_position + 3,
                    CastleSides::Long => king_from_position - 4,
                };
                let rook_to_position = match side {
                    CastleSides::Short => king_from_position + 1,
                    CastleSides::Long => king_from_position - 1,
                };

                // Move our rook over.
                let rook_bitboard_index =
                    our_piece_bitboard_offset + PieceType::Rook.bitboard_index();
                self.piece_bitboards[rook_bitboard_index] = pop_bit(
                    self.piece_bitboards[rook_bitboard_index],
                    rook_from_position,
                );
                self.piece_bitboards[rook_bitboard_index] =
                    set_bit(self.piece_bitboards[rook_bitboard_index], rook_to_position);

                self.zobrist_hash ^=
                    self.bitboard_constants.zobrist_table[rook_bitboard_index][rook_from_position];
                self.zobrist_hash ^=
                    self.bitboard_constants.zobrist_table[rook_bitboard_index][rook_to_position];

                // Update our occupancies.
                self.occupancy_bitboards[our_occupancies_index] = pop_bit(
                    self.occupancy_bitboards[our_occupancies_index],
                    rook_from_position,
                );
                self.occupancy_bitboards[our_occupancies_index] = set_bit(
                    self.occupancy_bitboards[our_occupancies_index],
                    rook_to_position,
                );

                // Update global occupancies.
                self.occupancy_bitboards[2] =
                    pop_bit(self.occupancy_bitboards[2], rook_from_position);
                self.occupancy_bitboards[2] =
                    set_bit(self.occupancy_bitboards[2], rook_to_position);
            }
        }

        // Forfeiting castling rights.
        if this_move.removes_white_castling_rights_short == Some(true) {
            self.can_white_castle_short = false;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[0];
        }
        if this_move.removes_white_castling_rights_long == Some(true) {
            self.can_white_castle_long = false;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[1];
        }
        if this_move.removes_black_castling_rights_short == Some(true) {
            self.can_black_castle_short = false;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[2];
        }
        if this_move.removes_black_castling_rights_long == Some(true) {
            self.can_black_castle_long = false;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[3];
        }

        // Only needed if we are capturing.
        self.en_passant_target = this_move.next_en_passant_target_coord;

        // Update zobrist hash based on en-passant file.
        match this_move.next_en_passant_target_coord {
            Some(square) => {
                self.zobrist_hash ^= self.bitboard_constants.zobrist_en_passant[square % 8];
            }
            None => (),
        }
        match this_move.last_en_passant_target_coord {
            Some(square) => {
                self.zobrist_hash ^= self.bitboard_constants.zobrist_en_passant[square % 8];
            }
            None => (),
        }

        // Important for checking if move is illegal.
        self.white_to_move = !self.white_to_move;
        self.zobrist_hash ^= self.bitboard_constants.zobrist_to_move;

        if update_legal_moves {
            // Update our moves!
            self.set_legal_moves(None);
        }

        // Used to find bugs.
        //self.debug_verify_board_state(this_move, debug_initial_game_state, "Make Move");
    }

    pub fn unmake_move(&mut self, this_move: &Move) {
        //let debug_initial_game_state = self.clone();

        let source_piece = this_move
            .from_piece_type
            .expect("This should always be here.");

        // Handle generic captures, and en-passant captures.
        let our_color: Color;
        let their_color: Color;

        // If I'm unmaking white's move, I'm white.
        if !self.white_to_move {
            our_color = Color::White;
            their_color = Color::Black;
        } else {
            our_color = Color::Black;
            their_color = Color::White;
        }

        let our_piece_bitboard_offset: usize = our_color.piece_bitboard_offset();
        let our_piece_bitboard_index: usize =
            our_piece_bitboard_offset + source_piece.bitboard_index();
        let our_occupancies_index: usize = our_color.occupancy_bitboard_index();
        let their_piece_bitboard_offset: usize = their_color.piece_bitboard_offset();
        let their_occupancies_index: usize = their_color.occupancy_bitboard_index();

        // Place the piece back it's starting square.
        match source_piece {
            PieceType::Pawn => {
                self.piece_bitboards[our_piece_bitboard_index] = set_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.from_square,
                );
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.from_square];

                // Special logic for pawn DEMOTION.
                match this_move.pawn_promoting_to {
                    Some(piece_promoted_to) => {
                        match piece_promoted_to {
                            PieceType::Queen => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = pop_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Rook => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = pop_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Bishop => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = pop_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            PieceType::Knight => {
                                self.piece_bitboards[our_piece_bitboard_offset
                                    + piece_promoted_to.bitboard_index()] = pop_bit(
                                    self.piece_bitboards[our_piece_bitboard_offset
                                        + piece_promoted_to.bitboard_index()],
                                    this_move.to_square,
                                )
                            }
                            _ => panic!("Tried to promote to an illegal piece."),
                        }
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [our_piece_bitboard_offset + piece_promoted_to.bitboard_index()]
                            [this_move.to_square];
                    }
                    None => {
                        self.piece_bitboards[our_piece_bitboard_index] = pop_bit(
                            self.piece_bitboards[our_piece_bitboard_index],
                            this_move.to_square,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [our_piece_bitboard_index][this_move.to_square];
                    }
                }
            }

            // Every other piece, remove it from the destination, place it at the source.
            _ => {
                self.piece_bitboards[our_piece_bitboard_index] = pop_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.to_square,
                );
                self.piece_bitboards[our_piece_bitboard_index] = set_bit(
                    self.piece_bitboards[our_piece_bitboard_index],
                    this_move.from_square,
                );
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.to_square];
                self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                    [our_piece_bitboard_index][this_move.from_square];
            }
        }

        // Update our occupancies.
        self.occupancy_bitboards[our_occupancies_index] = pop_bit(
            self.occupancy_bitboards[our_occupancies_index],
            this_move.to_square,
        );
        self.occupancy_bitboards[our_occupancies_index] = set_bit(
            self.occupancy_bitboards[our_occupancies_index],
            this_move.from_square,
        );

        // Update all occupancies, source piece always moves.
        self.occupancy_bitboards[2] = set_bit(self.occupancy_bitboards[2], this_move.from_square);

        // Figure out if we are capturing.
        let is_capture = this_move.to_piece_type.is_some();
        if !is_capture {
            self.occupancy_bitboards[2] = pop_bit(self.occupancy_bitboards[2], this_move.to_square);
        } else {
            let their_piece = this_move
                .to_piece_type
                .expect("Should be here, thanks to guard.");
            let their_piece_bitboard_index =
                their_piece.bitboard_index() + their_piece_bitboard_offset;

            // Place their piece on the square; and update their occupancies.
            match their_piece {
                // For pawn captures, handle en-passant.
                PieceType::Pawn => {
                    if this_move.is_en_passant_capture {
                        // Remove our pawn from the target square. Normal captures do not need to update this, but en-passant does.
                        self.occupancy_bitboards[2] =
                            pop_bit(self.occupancy_bitboards[2], this_move.to_square);
                        let en_passant_target_pawn_index: usize = match their_color {
                            Color::White => this_move.to_square - 8,
                            Color::Black => this_move.to_square + 8,
                        };

                        // Add their pawn we captured en-passant.
                        self.piece_bitboards
                            [their_piece_bitboard_offset + their_piece.bitboard_index()] = set_bit(
                            self.piece_bitboards
                                [their_piece_bitboard_offset + their_piece.bitboard_index()],
                            en_passant_target_pawn_index,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [their_piece_bitboard_offset + their_piece.bitboard_index()]
                            [en_passant_target_pawn_index];

                        // Add their occupancy.
                        self.occupancy_bitboards[their_occupancies_index] = set_bit(
                            self.occupancy_bitboards[their_occupancies_index],
                            en_passant_target_pawn_index,
                        );

                        // Add global occupancy.
                        self.occupancy_bitboards[2] =
                            set_bit(self.occupancy_bitboards[2], en_passant_target_pawn_index);
                    } else {
                        // Add that piece from the board.
                        self.piece_bitboards[their_piece_bitboard_index] = set_bit(
                            self.piece_bitboards[their_piece_bitboard_index],
                            this_move.to_square,
                        );
                        self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                            [their_piece_bitboard_index][this_move.to_square];

                        // Update their occupancies.
                        self.occupancy_bitboards[their_occupancies_index] = set_bit(
                            self.occupancy_bitboards[their_occupancies_index],
                            this_move.to_square,
                        );
                    }
                }

                // For all non-pawn captures...
                _ => {
                    // Add that piece from the board.
                    self.piece_bitboards[their_piece_bitboard_index] = set_bit(
                        self.piece_bitboards[their_piece_bitboard_index],
                        this_move.to_square,
                    );
                    self.zobrist_hash ^= self.bitboard_constants.zobrist_table
                        [their_piece_bitboard_index][this_move.to_square];

                    // Update their occupancies.
                    self.occupancy_bitboards[their_occupancies_index] = set_bit(
                        self.occupancy_bitboards[their_occupancies_index],
                        this_move.to_square,
                    );
                }
            }
        }

        // Lastly, handle castling.
        match this_move.castle_side {
            None => (),
            Some(side) => {
                let king_from_position = this_move.from_square;
                let rook_from_position = match side {
                    CastleSides::Short => king_from_position + 3,
                    CastleSides::Long => king_from_position - 4,
                };
                let rook_to_position = match side {
                    CastleSides::Short => king_from_position + 1,
                    CastleSides::Long => king_from_position - 1,
                };

                // Move our rook back.
                let rook_bitboard_index =
                    our_piece_bitboard_offset + PieceType::Rook.bitboard_index();
                self.piece_bitboards[rook_bitboard_index] = set_bit(
                    self.piece_bitboards[rook_bitboard_index],
                    rook_from_position,
                );
                self.piece_bitboards[rook_bitboard_index] =
                    pop_bit(self.piece_bitboards[rook_bitboard_index], rook_to_position);

                self.zobrist_hash ^=
                    self.bitboard_constants.zobrist_table[rook_bitboard_index][rook_from_position];
                self.zobrist_hash ^=
                    self.bitboard_constants.zobrist_table[rook_bitboard_index][rook_to_position];

                // Update our occupancies.
                self.occupancy_bitboards[our_occupancies_index] = set_bit(
                    self.occupancy_bitboards[our_occupancies_index],
                    rook_from_position,
                );
                self.occupancy_bitboards[our_occupancies_index] = pop_bit(
                    self.occupancy_bitboards[our_occupancies_index],
                    rook_to_position,
                );

                // Update global occupancies.
                self.occupancy_bitboards[2] =
                    set_bit(self.occupancy_bitboards[2], rook_from_position);
                self.occupancy_bitboards[2] =
                    pop_bit(self.occupancy_bitboards[2], rook_to_position);
            }
        }

        // Grant castling rights back if we forfeited them.
        if this_move.removes_white_castling_rights_short == Some(true) {
            self.can_white_castle_short = true;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[0];
        }
        if this_move.removes_white_castling_rights_long == Some(true) {
            self.can_white_castle_long = true;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[1];
        }
        if this_move.removes_black_castling_rights_short == Some(true) {
            self.can_black_castle_short = true;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[2];
        }
        if this_move.removes_black_castling_rights_long == Some(true) {
            self.can_black_castle_long = true;
            self.zobrist_hash ^= self.bitboard_constants.zobrist_castling_rights[3];
        }

        // Only needed if we are capturing.
        self.en_passant_target = this_move.last_en_passant_target_coord;

        // Update zobrist hash based on en-passant file.
        match this_move.next_en_passant_target_coord {
            Some(square) => {
                self.zobrist_hash ^= self.bitboard_constants.zobrist_en_passant[square % 8];
            }
            None => (),
        }
        match this_move.last_en_passant_target_coord {
            Some(square) => {
                self.zobrist_hash ^= self.bitboard_constants.zobrist_en_passant[square % 8];
            }
            None => (),
        }

        // Important for checking if move is illegal.
        self.white_to_move = !self.white_to_move;
        self.zobrist_hash ^= self.bitboard_constants.zobrist_to_move;

        // Debugging!
        //self.debug_verify_board_state(this_move, debug_initial_game_state, "Unmake move");
    }

    pub fn sort_moves(&self, moves: &mut Vec<Move>) {
        moves.sort_unstable_by_key(|m| {
            let mut move_evaluation: i64 = 0;
            let is_check = m.is_check == Some(true);
            let is_capture = m.to_piece_type.is_some();
            let our_color: Color;
            let their_color: Color;
            if self.white_to_move {
                our_color = Color::White;
                their_color = Color::Black;
            } else {
                our_color = Color::Black;
                their_color = Color::White;
            }

            // Smaller move value is better.
            if is_check {
                move_evaluation -= 10;
            }

            // Look at captures.
            if is_capture {
                move_evaluation -= 5;
            }

            /*
               The below checks actually slow down minimax a bit...
               but perhaps time will be saved again by looking down the right trees?
               Just theory crafting for now.
            */

            // If moving to a square we are attacking, that is good. (destination is defended)
            if self.is_square_attacked(m.to_square, &our_color) {
                move_evaluation -= 2;
            }

            // If moving to a square they are attacking, that is bad. (destination is attacked)
            if self.is_square_attacked(m.to_square, &their_color) {
                move_evaluation += 2;
            }

            // If moving from a square they are attacking, that is good. (escaping an attack)
            if self.is_square_attacked(m.from_square, &their_color) {
                move_evaluation -= 2;
            }

            return move_evaluation;
        });
    }

    pub fn is_king_attacked(&self, side_attacked: &Color) -> bool {
        let king_bitboard_index = 5 + side_attacked.piece_bitboard_offset();
        let king_square = get_lsb_index(self.piece_bitboards[king_bitboard_index])
            .expect("King must be on board.");
        return match side_attacked {
            Color::White => self.is_square_attacked(king_square, &Color::Black),
            Color::Black => self.is_square_attacked(king_square, &Color::White),
        };
    }

    pub fn is_checkmate(&self) -> bool {
        // If you have a legal move, you are not in checkmate.
        if self.legal_moves.len() != 0 {
            return false;
        }

        let our_color: Color;
        if self.white_to_move {
            our_color = Color::White;
        } else {
            our_color = Color::Black;
        }

        return self.is_king_attacked(&our_color);
    }

    pub fn is_stalemate(&self) -> bool {
        // If you have a legal move, you are not in checkmate.
        if self.legal_moves.len() != 0 {
            return false;
        }

        let our_color: Color;
        if self.white_to_move {
            our_color = Color::White;
        } else {
            our_color = Color::Black;
        }

        return !self.is_king_attacked(&our_color);
    }

    pub fn evaluate_board(&self) -> i64 {
        // Variables shared by both functions.
        let mut square: usize;
        let mut occupancies: u64;
        let mut white_piece_value_total: i64 = 0;
        let mut black_piece_value_total: i64 = 0;

        // Add up white pieces.
        occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
        while occupancies != 0 {
            square = get_lsb_index(occupancies).expect("Guard clause.");
            let (piece_wrapped, _) = self.get_piece_at_square(square);
            let piece = piece_wrapped.expect("Not empty (white piece).");
            white_piece_value_total += piece.piece_base_value();
            white_piece_value_total += piece.piece_happy_square_value(square, true);
            occupancies = pop_bit(occupancies, square)
        }

        // Add up black pieces.
        occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
        while occupancies != 0 {
            square = get_lsb_index(occupancies).expect("Guard clause.");
            let (piece_wrapped, _) = self.get_piece_at_square(square);

            if piece_wrapped.is_none() {
                println!("Expecting piece at square {}; but did not find it.", square);
                println!("Position:");
                self.print_board();

                println!("Black occupancies:");
                print_bitboard(self.occupancy_bitboards[1]);
            }

            let piece = piece_wrapped.expect("Not empty (black piece).");
            black_piece_value_total += piece.piece_base_value();
            black_piece_value_total += piece.piece_happy_square_value(square, false);
            occupancies = pop_bit(occupancies, square)
        }

        // Add up the pieces, return the sum?
        return white_piece_value_total - black_piece_value_total;
    }

    // Meant for users/bots to pick a move, so it is populated with all the data we need.
    pub fn choose_move_from_legal_move(&mut self, this_move: &Move) -> Option<Move> {
        let moves = self.get_legal_moves();

        for m in moves.iter() {
            if m == this_move {
                return Some(*m);
            }
        }

        return None;
    }

    pub fn set_legal_moves(&mut self, moves: Option<Vec<Move>>) {
        self.legal_moves.clear();
        self.legal_moves = match moves {
            Some(m) => m,
            None => self.get_legal_moves(),
        };
    }

    pub fn get_legal_moves(&mut self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        let mut possible_moves = self.get_psuedo_legal_moves();
        let our_side: &Color;
        let their_side: &Color;

        if self.white_to_move {
            our_side = &Color::White;
            their_side = &Color::Black;
        } else {
            our_side = &Color::Black;
            their_side = &Color::White;
        }

        // Try the move, drop it if it's illegal.
        for this_move in possible_moves.iter_mut() {
            // How does this move impact castling rights?
            this_move.removes_white_castling_rights_short = Some(false);
            this_move.removes_white_castling_rights_long = Some(false);
            this_move.removes_black_castling_rights_short = Some(false);
            this_move.removes_black_castling_rights_long = Some(false);

            // Revoke our castling rights based on our move.
            match our_side {
                Color::White => {
                    if self.can_white_castle_short {
                        // If we are moving the king, remove this right.
                        if this_move.from_piece_type == Some(PieceType::King) {
                            this_move.removes_white_castling_rights_short = Some(true);
                        }
                        // If we are moving our rook from it's starting square, remove this right.
                        else if this_move.from_piece_type == Some(PieceType::Rook)
                            && this_move.from_square == 63
                        {
                            this_move.removes_white_castling_rights_short = Some(true);
                        }
                    }

                    if self.can_white_castle_long {
                        // If we are moving the king, remove this right.
                        if this_move.from_piece_type == Some(PieceType::King) {
                            this_move.removes_white_castling_rights_long = Some(true);
                        }
                        // If we are moving our rook from it's starting square, remove this right.
                        else if this_move.from_piece_type == Some(PieceType::Rook)
                            && this_move.from_square == 56
                        {
                            this_move.removes_white_castling_rights_long = Some(true);
                        }
                    }
                }
                Color::Black => {
                    if self.can_black_castle_short {
                        // If we are moving the king, remove this right.
                        if this_move.from_piece_type == Some(PieceType::King) {
                            this_move.removes_black_castling_rights_short = Some(true);
                        }
                        // If we are moving our rook from it's starting square, remove this right.
                        else if this_move.from_piece_type == Some(PieceType::Rook)
                            && this_move.from_square == 7
                        {
                            this_move.removes_black_castling_rights_short = Some(true);
                        }
                    }

                    if self.can_black_castle_long {
                        // If we are moving the king, remove this right.
                        if this_move.from_piece_type == Some(PieceType::King) {
                            this_move.removes_black_castling_rights_long = Some(true);
                        }
                        // If we are moving our rook from it's starting square, remove this right.
                        else if this_move.from_piece_type == Some(PieceType::Rook)
                            && this_move.from_square == 0
                        {
                            this_move.removes_black_castling_rights_long = Some(true);
                        }
                    }
                }
            }

            // Handle the edge case where we are capturing opponents rook on it's starting square. We need to revoke rights.
            if this_move.to_piece_type == Some(PieceType::Rook) {
                match their_side {
                    Color::Black => {
                        // If black can castle short, but we are capturing the rook on it's starting square; revoke.
                        if self.can_black_castle_short && this_move.to_square == 7 {
                            this_move.removes_black_castling_rights_short = Some(true);
                        }
                        // If black can castle long, but we are capturing the rook on it's starting square; revoke.
                        else if self.can_black_castle_long && this_move.to_square == 0 {
                            this_move.removes_black_castling_rights_long = Some(true);
                        }
                    }
                    Color::White => {
                        // If white can castle short, but we are capturing the rook on it's starting square; revoke.
                        if self.can_white_castle_short && this_move.to_square == 63 {
                            this_move.removes_white_castling_rights_short = Some(true);
                        }
                        // If white can castle long, but we are capturing the rook on it's starting square; revoke.
                        else if self.can_white_castle_long && this_move.to_square == 56 {
                            this_move.removes_white_castling_rights_long = Some(true);
                        }
                    }
                }
            }

            self.make_move(this_move, false);

            // Does the move put us in check?
            if !self.is_king_attacked(our_side) {
                // Does it put them in check?
                this_move.is_check = Some(self.is_king_attacked(their_side));
                moves.push(*this_move);
            }

            self.unmake_move(this_move);
        }

        self.sort_moves(&mut moves);

        return moves;
    }

    // Will generate moves that put self in check.
    pub fn get_psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        // Get all the moves.
        moves.append(&mut self.get_moves_slider(PieceType::Queen));
        moves.append(&mut self.get_moves_slider(PieceType::Rook));
        moves.append(&mut self.get_moves_slider(PieceType::Bishop));
        moves.append(&mut self.get_moves_knight());
        moves.append(&mut self.get_moves_king());
        moves.append(&mut self.get_moves_pawns());

        return moves;
    }

    pub fn print_legal_moves(&self) {
        for m in self.legal_moves.iter() {
            print!("{} ", m.move_to_str());
        }
        if self.legal_moves.len() == 0 {
            print!("There are no legal moves...");
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
        let mut to_piece_type: Option<PieceType>;

        let all_occupancies: u64 = self.occupancy_bitboards[2];
        let their_occupancies: u64;
        let our_piece_bitboard_offset: usize;

        if self.white_to_move {
            their_occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
            our_piece_bitboard_offset = Color::White.piece_bitboard_offset();
        } else {
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            our_piece_bitboard_offset = Color::Black.piece_bitboard_offset();
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

        while slider_pieces != 0 {
            source_square = get_lsb_index(slider_pieces).expect("This should not happen.");

            // Get moves and captures seperately.
            match slider_piece_type {
                PieceType::Queen => {
                    slider_piece_attacks = self.get_queen_attacks(source_square, all_occupancies);
                }
                PieceType::Rook => {
                    slider_piece_attacks = self.get_rook_attacks(source_square, all_occupancies);
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
                    is_en_passant_capture: false,
                    is_check: None,
                    pawn_promoting_to: None,
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
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
                    is_en_passant_capture: false,
                    is_check: None,
                    pawn_promoting_to: None,
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
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
            knights = self.piece_bitboards
                [Color::White.piece_bitboard_offset() + PieceType::Knight.bitboard_index()];
        } else {
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            knights = self.piece_bitboards
                [Color::Black.piece_bitboard_offset() + PieceType::Knight.bitboard_index()];
        }

        while knights != 0 {
            source_square = get_lsb_index(knights).expect("This should not happen.");

            // Get moves and captures seperately.
            quiet_moves = self.bitboard_constants.knight_attacks[source_square]
                & (!self.occupancy_bitboards[2]);
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
                    is_en_passant_capture: false,
                    pawn_promoting_to: None,
                    castle_side: None,
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
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
                    is_en_passant_capture: false,
                    pawn_promoting_to: None,
                    castle_side: None,
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
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
        let mut to_piece_type: Option<PieceType>;

        let their_color: &Color;
        let their_occupancies: u64;
        let can_castle_long: bool;
        let can_castle_short: bool;
        let king_starting_square: usize;
        if self.white_to_move {
            their_color = &Color::Black;
            their_occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
            bitboard = self.piece_bitboards
                [Color::White.piece_bitboard_offset() + PieceType::King.bitboard_index()];
            can_castle_short = self.can_white_castle_short;
            can_castle_long = self.can_white_castle_long;
            king_starting_square = 60;
        } else {
            their_color = &Color::White;
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            bitboard = self.piece_bitboards
                [Color::Black.piece_bitboard_offset() + PieceType::King.bitboard_index()];
            can_castle_short = self.can_black_castle_short;
            can_castle_long = self.can_black_castle_long;
            king_starting_square = 4;
        }

        if bitboard == 0 {
            return moves;
        }

        source_square = get_lsb_index(bitboard).expect("Guard before should handle this.");
        let mut quiet_moves =
            self.bitboard_constants.king_attacks[source_square] & (!self.occupancy_bitboards[2]);
        let mut attacks = self.bitboard_constants.king_attacks[source_square] & their_occupancies;

        // Moves
        while quiet_moves != 0 {
            target_square = get_lsb_index(quiet_moves).expect("Guard before should handle this.");
            moves.push(Move {
                from_square: source_square,
                from_piece_type: Some(PieceType::King),
                to_square: target_square,
                to_piece_type: None,
                is_check: None,
                last_en_passant_target_coord: self.en_passant_target,
                next_en_passant_target_coord: None,
                is_en_passant_capture: false,
                pawn_promoting_to: None,
                castle_side: None,
                removes_white_castling_rights_short: None,
                removes_white_castling_rights_long: None,
                removes_black_castling_rights_short: None,
                removes_black_castling_rights_long: None,
            });
            quiet_moves = pop_bit(quiet_moves, target_square);
        }

        // Attacks
        while attacks != 0 {
            target_square = get_lsb_index(attacks).expect("Guard before should handle this.");
            (to_piece_type, _) = self.get_piece_at_square(target_square);

            moves.push(Move {
                from_square: source_square,
                from_piece_type: Some(PieceType::King),
                to_square: target_square,
                to_piece_type: to_piece_type,
                is_check: None,
                last_en_passant_target_coord: self.en_passant_target,
                next_en_passant_target_coord: None,
                is_en_passant_capture: false,
                pawn_promoting_to: None,
                castle_side: None,
                removes_white_castling_rights_short: None,
                removes_white_castling_rights_long: None,
                removes_black_castling_rights_short: None,
                removes_black_castling_rights_long: None,
            });

            attacks = pop_bit(attacks, target_square);
        }

        // Castling
        if can_castle_short {
            // 1. Make sure squares are empty.
            let squares_should_be_empty =
                set_bit(0, king_starting_square + 1) | set_bit(0, king_starting_square + 2);

            // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
            let is_intermediary_square_attacked =
                self.is_square_attacked(king_starting_square + 1, their_color);

            // If both conditions are met, we can castle.
            target_square = king_starting_square + 2;
            if (squares_should_be_empty & self.occupancy_bitboards[2]) == 0
                && !is_intermediary_square_attacked
            {
                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(PieceType::King),
                    to_square: target_square,
                    to_piece_type: None,
                    is_check: None,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    is_en_passant_capture: false,
                    pawn_promoting_to: None,
                    castle_side: Some(CastleSides::Short),
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
                });
            }
        }

        if can_castle_long {
            // 1. Make sure squares are empty.
            let squares_should_be_empty = set_bit(0, king_starting_square - 1)
                | set_bit(0, king_starting_square - 2)
                | set_bit(0, king_starting_square - 3);

            // 2. Make sure intermediary square is not attacked. Our final check for pins will handle checking the destination square.
            let is_intermediary_square_attacked =
                self.is_square_attacked(king_starting_square - 1, their_color);

            // If both conditions are met, we can castle.
            target_square = king_starting_square - 2;
            if (squares_should_be_empty & self.occupancy_bitboards[2]) == 0
                && !is_intermediary_square_attacked
            {
                moves.push(Move {
                    from_square: source_square,
                    from_piece_type: Some(PieceType::King),
                    to_square: target_square,
                    to_piece_type: None,
                    is_check: None,
                    last_en_passant_target_coord: self.en_passant_target,
                    next_en_passant_target_coord: None,
                    is_en_passant_capture: false,
                    pawn_promoting_to: None,
                    castle_side: Some(CastleSides::Long),
                    removes_white_castling_rights_short: None,
                    removes_white_castling_rights_long: None,
                    removes_black_castling_rights_short: None,
                    removes_black_castling_rights_long: None,
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
        let all_occupancies: u64 = self.occupancy_bitboards[2];
        let mut to_piece_type: Option<PieceType>;
        if self.white_to_move {
            our_color = Color::White;
            their_occupancies = self.occupancy_bitboards[Color::Black.occupancy_bitboard_index()];
            bitboard = self.piece_bitboards
                [Color::White.piece_bitboard_offset() + PieceType::Pawn.bitboard_index()];
            pawn_move_offset = -8;
            promotion_rank_lower = 0;
            promotion_rank_upper = 7;
            our_starting_rank_lower = 48;
            our_starting_rank_upper = 55;
        } else {
            our_color = Color::Black;
            their_occupancies = self.occupancy_bitboards[Color::White.occupancy_bitboard_index()];
            bitboard = self.piece_bitboards
                [Color::Black.piece_bitboard_offset() + PieceType::Pawn.bitboard_index()];
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
            let mut is_occupied = get_bit(all_occupancies, target_square) != 0;
            if !is_occupied {
                // Check for promotions (no capture).
                if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: None,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Queen),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: None,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Rook),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: None,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Bishop),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: None,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Knight),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                } else {
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: None,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: None,
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });

                    // Check for the double move.
                    target_square =
                        (source_square as i32 + pawn_move_offset + pawn_move_offset) as usize;
                    is_occupied = get_bit(all_occupancies, target_square) != 0;

                    // If pawn is on the 2nd rank, it can move two tiles.
                    if source_square >= our_starting_rank_lower
                        && source_square <= our_starting_rank_upper
                        && !is_occupied
                    {
                        moves.push(Move {
                            from_square: source_square,
                            from_piece_type: Some(PieceType::Pawn),
                            to_square: target_square,
                            to_piece_type: None,
                            is_check: None,
                            last_en_passant_target_coord: self.en_passant_target,
                            next_en_passant_target_coord: Some(
                                (source_square as i32 + pawn_move_offset) as usize,
                            ),
                            is_en_passant_capture: false,
                            pawn_promoting_to: None,
                            castle_side: None,
                            removes_white_castling_rights_short: None,
                            removes_white_castling_rights_long: None,
                            removes_black_castling_rights_short: None,
                            removes_black_castling_rights_long: None,
                        });
                    }
                }
            }

            // Handles captures (non-en-passant).
            attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square]
                & their_occupancies;
            while attacks != 0 {
                target_square = get_lsb_index(attacks).expect("Should not be empty.");
                (to_piece_type, _) = self.get_piece_at_square(target_square);
                if target_square >= promotion_rank_lower && target_square <= promotion_rank_upper {
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: to_piece_type,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Queen),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: to_piece_type,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Rook),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: to_piece_type,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Bishop),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: to_piece_type,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: Some(PieceType::Knight),
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                } else {
                    moves.push(Move {
                        from_square: source_square,
                        from_piece_type: Some(PieceType::Pawn),
                        to_square: target_square,
                        to_piece_type: to_piece_type,
                        is_check: None,
                        last_en_passant_target_coord: self.en_passant_target,
                        next_en_passant_target_coord: None,
                        is_en_passant_capture: false,
                        pawn_promoting_to: None,
                        castle_side: None,
                        removes_white_castling_rights_short: None,
                        removes_white_castling_rights_long: None,
                        removes_black_castling_rights_short: None,
                        removes_black_castling_rights_long: None,
                    });
                }
                attacks = pop_bit(attacks, target_square);
            }

            // Handles captures (en-passant)
            match self.en_passant_target {
                Some(s) => {
                    attacks = self.bitboard_constants.pawn_attacks[our_color.idx()][source_square]
                        & set_bit(0, s);

                    if attacks != 0 {
                        target_square = get_lsb_index(attacks).expect("This should not be empty.");
                        moves.push(Move {
                            from_square: source_square,
                            from_piece_type: Some(PieceType::Pawn),
                            to_square: target_square,
                            to_piece_type: Some(PieceType::Pawn),
                            is_check: None,
                            last_en_passant_target_coord: self.en_passant_target,
                            next_en_passant_target_coord: None,
                            is_en_passant_capture: true,
                            pawn_promoting_to: None,
                            castle_side: None,
                            removes_white_castling_rights_short: None,
                            removes_white_castling_rights_long: None,
                            removes_black_castling_rights_short: None,
                            removes_black_castling_rights_long: None,
                        });
                    }
                }
                _ => (),
            }

            // Empty the board! and go next.
            bitboard = pop_bit(bitboard, source_square);
        }

        return moves;
    }

    pub fn play_game_vs_bot(&mut self) {
        //self.import_fen(INITIAL_GAME_STATE_FEN);
        //self.import_fen("rnb1kbnr/pppp1ppp/11111111/1111p111/1111PP1q/111111P1/PPPP111P/RNBQKBNR b");
        //self.import_fen("rnbqkbnr/pppp1ppp/8/4p3/5PP1/8/PPPPP2P/RNBQKBNR b"); // M1 for black.
        //self.import_fen("rnbqkbnr/p1pppppp/8/1p3P2/8/8/PPPPP1PP/RNBQKBNR b KQkq - 0 2"); // Testing for en-passant.
        //self.import_fen("rnbqkbnr/p1pp1ppp/8/1p2pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3"); // Direct capture allowed.
        //self.import_fen("rnbqkbnr/pppppppp/8/8/4B3/4N3/PPPPPPPP/RNBQK2R w KQkq - 0 1"); // Can castle short?
        //self.import_fen("rnbqkbnr/pppppppp/8/8/1QN1B3/2B1N3/PPPPPPPP/R3K2R w KQkq - 0 1"); // Can castle long and short.
        //self.import_fen("rnbqk3/1pppp1P1/7P/5P2/1QN1B3/1PB1N3/pPPPP3/4K2R w Kq - 0 1"); // Pawn promotion.
        //self.update_legal_moves();
        println!("Starting a new game.");

        let mut iter_counter: i32 = 0;
        loop {
            // Print the board.
            self.print_board();

            // See if game is over.
            if self.legal_moves.len() == 0 {
                //self.state.print_game_state();
                //self.print_debug_game_state();
                println!("Game over, no legal moves.");
                break;
            }

            // Player move.
            println!("It is your turn. Enter a move.");

            // Read the user input.
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to readline. Not sure what went wrong.");

            // Remove endline.
            input = String::from(input.trim());

            // TEMPORARY DEBUG OUTPUT FEN
            if input == "export" {
                //println!("{}", self.export_fen());
                println!("Not implemented yet.");
                continue;
            }

            // TEMPORARY DEBUG OUTPUT GAME STATE
            if input == "debug" {
                // self.print_debug_game_state();
                println!("Not implemented yet.");
                continue;
            }

            let user_move_raw = match Move::str_to_move(&input) {
                Ok(m) => m,
                Err(msg) => {
                    println!("{}", msg);
                    continue;
                }
            };

            // Grab the move from the game, it has more data (capture, en-passant, etc).
            let user_move: Option<Move> = self.choose_move_from_legal_move(&user_move_raw);

            match user_move {
                Some(m) => self.make_move(&m, true),
                None => {
                    println!("That is not one of your legal moves. Try again.");
                    self.print_legal_moves();
                    continue;
                }
            }

            println!("Made the move...");
            self.print_board();
            //self.print_debug_game_state();
            if self.legal_moves.len() == 0 {
                println!("Game over, no more legal moves.");
                break;
            }

            // Let the bot make a move.
            let bot_move = self.get_bot_move();
            println!("Bot is playing {}", bot_move.move_to_str());
            self.make_move(&bot_move, true);

            // Temporary guard for oopsies...
            iter_counter += 1;
            if iter_counter > 1000 {
                panic!("Dev likely did something wrong, hit 1000 iterations.");
            }
        }
    }

    pub fn get_bot_move(&mut self) -> Move {
        if self.legal_moves.len() == 0 {
            panic!("Something has gone wrong, called get_bot_move when no legal moves were available...");
        }

        //let (evaluation, best_move) = self.minimax(4, std::i64::MIN, std::i64::MAX);
        let (evaluation, best_move) = self.iterative_deepening_minimax();

        return best_move.unwrap();
    }

    pub fn get_bot_move_debug(&mut self) -> Move {
        if self.legal_moves.len() == 0 {
            panic!("Something has gone wrong, called get_bot_move when no legal moves were available...");
        }

        let (evaluation, best_move) = self.iterative_deepening_minimax();

        println!("Best move evaluation {evaluation}");

        return best_move.unwrap();
    }

    pub fn iterative_deepening_minimax(&mut self) -> (i64, Option<Move>) {
        let start_time = std::time::SystemTime::now();
        let mut best_evaluation: i64 = 0;
        let mut best_move: Option<Move> = None;
        let mut search_depth = 1;

        // Iteratively deepen...
        loop {
            //println!("Currently searching depth {search_depth}");

            // Search at the current depth.
            (best_evaluation, best_move) = self.minimax(search_depth, std::i64::MIN, std::i64::MAX);

            // See how long that last operation took. If it was too long, stop the search.
            let time_spent_ms = std::time::SystemTime::now()
                .duration_since(start_time)
                .expect("Time went back?")
                .as_millis();
            if time_spent_ms >= 5_000 {
                break;
            }

            //println!("Currently spent {time_spent_ms}ms");

            // Otherwise, increase our depth and continue!
            search_depth += 1;
        }

        //println!("This search reached depth {search_depth}");

        // Return the best moves we found.
        return (best_evaluation, best_move);
    }

    pub fn minimax(&mut self, depth: u32, mut alpha: i64, mut beta: i64) -> (i64, Option<Move>) {
        self.debug_minimax_calls += 1;

        let zobrist_hash_index = self.zobrist_hash % 10_000;

        if self.transposition_table.contains_key(&zobrist_hash_index) {
            let entry = self
                .transposition_table
                .get(&zobrist_hash_index)
                .expect("Guard clause filters this.");
            if entry.depth >= depth && self.zobrist_hash == entry.zobrist_hash {
                //println!("{}Cache hit at good depth!", debug_depth_to_tabs(depth));
                match entry.node_type {
                    TranspositionTableNodeType::Exact => {
                        return (entry.evaluation, entry.best_move);
                    }
                    TranspositionTableNodeType::LowerBound => {
                        if entry.evaluation > alpha {
                            return (entry.evaluation, entry.best_move);
                        }
                    }
                    TranspositionTableNodeType::UpperBound => {
                        if entry.evaluation < beta {
                            return (entry.evaluation, entry.best_move);
                        }
                    }
                }
            }
        }

        if self.legal_moves.len() == 0 {
            if self.white_to_move {
                if self.is_king_attacked(&Color::White) {
                    return (std::i64::MIN, None);
                } else {
                    return (0, None);
                }
            } else {
                if self.is_king_attacked(&Color::Black) {
                    return (std::i64::MAX, None);
                } else {
                    return (0, None);
                }
            }
        }

        if depth == 0 {
            return (self.evaluate_board(), None);
        }

        // Clone legal moves? Bad?
        let temp_legal_move_clone = self.legal_moves.clone();

        let mut best_evaluation: i64;
        let mut best_move: Option<Move> = Some(temp_legal_move_clone[0]); // Assume first move is best. Important if all moves lead to mate.
        let mut temp_evaluation: i64;

        if self.white_to_move {
            best_evaluation = std::i64::MIN;
            for legal_move in temp_legal_move_clone.iter() {
                // Make the move.
                self.make_move(legal_move, true);

                // Get the evaluation of that position.
                (temp_evaluation, _) = self.minimax(depth - 1, alpha, beta);

                // See if it's better.
                if temp_evaluation > best_evaluation {
                    best_evaluation = temp_evaluation;
                    best_move = Some(*legal_move);
                }

                // Prune.
                if best_evaluation > beta {
                    self.unmake_move(legal_move);
                    break;
                }

                // Undo the move, track alpha.
                self.unmake_move(legal_move);
                alpha = i64::max(alpha, best_evaluation);
            }
        } else {
            best_evaluation = std::i64::MAX;
            for legal_move in temp_legal_move_clone.iter() {
                // Make the move.
                self.make_move(legal_move, true);

                // Get the evaluation of that position.
                (temp_evaluation, _) = self.minimax(depth - 1, alpha, beta);

                // See if it's better.
                if temp_evaluation < best_evaluation {
                    best_evaluation = temp_evaluation;
                    best_move = Some(*legal_move);
                }

                // Prune.
                if best_evaluation < alpha {
                    self.unmake_move(legal_move);
                    break;
                }

                // Undo the move, track beta.
                self.unmake_move(legal_move);
                beta = i64::min(beta, best_evaluation);
            }
        }

        // Restore legal moves before exiting.
        self.set_legal_moves(Some(temp_legal_move_clone));

        // Find out transposition table node type.
        let node: TranspositionTableNodeType;
        if best_evaluation <= alpha {
            node = TranspositionTableNodeType::UpperBound;
        } else if best_evaluation >= beta {
            node = TranspositionTableNodeType::LowerBound;
        } else {
            node = TranspositionTableNodeType::Exact;
        }

        // Update transposition table.
        let time = std::time::SystemTime::now();
        self.transposition_table.insert(
            zobrist_hash_index,
            TranspositionTableEntry {
                zobrist_hash: self.zobrist_hash,
                best_move: best_move,
                depth: depth,
                node_type: node,
                evaluation: best_evaluation,
                age: time
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went back?")
                    .as_millis(),
            },
        );
        //println!("{}Set data in transposition table. Minimax call: {}", debug_depth_to_tabs(depth), self.debug_minimax_calls);

        return (best_evaluation, best_move);
    }

    pub fn minimax_debug(
        &mut self,
        depth: u32,
        mut alpha: i64,
        mut beta: i64,
    ) -> (i64, Option<Move>) {
        // Debugging!
        self.debug_minimax_calls += 1;

        if self.legal_moves.len() == 0 {
            if self.white_to_move {
                if self.is_king_attacked(&Color::White) {
                    return (std::i64::MIN, None);
                } else {
                    return (0, None);
                }
            } else {
                if self.is_king_attacked(&Color::Black) {
                    return (std::i64::MAX, None);
                } else {
                    return (0, None);
                }
            }
        }

        if depth == 0 {
            return (self.evaluate_board(), None);
        }

        // Clone legal moves? Bad?
        let temp_legal_move_clone = self.legal_moves.clone();

        let mut best_evaluation: i64;
        let mut best_move: Option<Move> = None;
        let mut temp_evaluation: i64;

        if self.white_to_move {
            best_evaluation = std::i64::MIN;
            for legal_move in temp_legal_move_clone.iter() {
                // Make the move.
                self.make_move(legal_move, true);

                // Debug
                self.debug_mimimax_moves_made.push(*legal_move);

                // Get the evaluation of that position.
                (temp_evaluation, _) = self.minimax(depth - 1, alpha, beta);

                // See if it's better.
                if temp_evaluation > best_evaluation {
                    best_evaluation = temp_evaluation;
                    best_move = Some(*legal_move);
                }

                // Prune.
                if best_evaluation > beta {
                    self.unmake_move(legal_move);

                    // Debug
                    self.debug_mimimax_moves_made.pop();

                    break;
                }

                // Undo the move, track alpha.
                self.unmake_move(legal_move);

                // Debug
                self.debug_mimimax_moves_made.pop();

                alpha = i64::max(alpha, best_evaluation);
            }
        } else {
            best_evaluation = std::i64::MAX;
            for legal_move in temp_legal_move_clone.iter() {
                // Make the move.
                self.make_move(legal_move, true);

                // Debug
                self.debug_mimimax_moves_made.push(*legal_move);

                // Get the evaluation of that position.
                (temp_evaluation, _) = self.minimax(depth - 1, alpha, beta);

                // See if it's better.
                if temp_evaluation < best_evaluation {
                    best_evaluation = temp_evaluation;
                    best_move = Some(*legal_move);
                }

                // Prune.
                if best_evaluation < alpha {
                    self.unmake_move(legal_move);

                    // Debug
                    self.debug_mimimax_moves_made.pop();

                    break;
                }

                // Undo the move, track beta.
                self.unmake_move(legal_move);

                // Debug
                self.debug_mimimax_moves_made.pop();

                beta = i64::min(beta, best_evaluation);
            }
        }

        // Restore legal moves before exiting.
        //self.set_legal_moves(Some(temp_legal_move_clone));

        return (best_evaluation, best_move);
    }

    pub fn print_debug_game_state_str(&self) {
        self.print_board();
        println!("White to move?: {}", self.white_to_move);
        println!("Zobrist Hash: {}", self.zobrist_hash);
        println!(
            "Transposition Table Size: {}",
            self.transposition_table.len()
        );

        print!("En-Passant Target Square: ");
        match self.en_passant_target {
            Some(square) => print!("{}.\n", square_to_coord(square)),
            None => print!("None.\n"),
        }

        println!("Castling rights:");
        println!("\tWhite Castle Short? {}", self.can_white_castle_short);
        println!("\tWhite Castle Long? {}", self.can_white_castle_long);
        println!("\tBlack Castle Short? {}", self.can_black_castle_short);
        println!("\tBlack Castle Long? {}", self.can_black_castle_long);

        print!("Legal moves: ");
        self.print_legal_moves();

        println!("FEN: {}", self.export_fen());
    }
}
