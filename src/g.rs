use std::{array, io, panic};
use rand::seq::SliceRandom;
use crate::{
    constants::BOARD_SIZE
    , constants::INITIAL_GAME_STATE_FEN
    , coord::Coord
    , my_move::Move
    , piece::Piece
    , piece_type::PieceType
    , game_states::GameState
    , castle_sides::CastleSides
};

// Struct for the chessboard, which is a 2D array of Pieces
#[derive(Clone)]
pub struct Game {
    pub board: [[Piece; BOARD_SIZE]; BOARD_SIZE],
    pub white_to_move: bool,
    pub state: GameState,
    pub legal_moves: Vec<Move>,
    pub en_passant_target: Option<Coord>,

    // Is there a better way to store this data?
    pub can_white_castle_long: bool,
    pub can_white_castle_short: bool,
    pub can_black_castle_long: bool,
    pub can_black_castle_short: bool,

    pub full_move_count: u32,
    pub half_move_count_non_pawn_non_capture: u32,
}

// Implement the Default trait for Board
impl Default for Game {
    fn default() -> Self {
        // Use from_fn to initialize the 2D array
        Game {
            board: array::from_fn(|_i| array::from_fn(|_j| Piece::default())),
            white_to_move: true,
            state: GameState::InProgress,
            legal_moves: vec![],
            en_passant_target: None,

            can_white_castle_long: true,
            can_white_castle_short: true,
            can_black_castle_long: true,
            can_black_castle_short: true,

            full_move_count: 1,
            half_move_count_non_pawn_non_capture: 0,
        }
    }
}

// Adds game specific methods to the game struct.
impl Game {
    pub fn print_board(&self) {
        println!("    A   B   C   D   E   F   G   H");
        println!("  |---|---|---|---|---|---|---|---|");
        for (y, row) in self.board.iter().enumerate() {
            print!("{} |", BOARD_SIZE - y);
            for piece in row.iter() {
                let display: char = PieceType::to_char(piece.piece_type, piece.white);

                print!(" {} |", display);
            }
            print!(" {}", BOARD_SIZE - y);
            println!();
            println!("  |---|---|---|---|---|---|---|---|");
        }
        println!("    A   B   C   D   E   F   G   H");
    }

    pub fn debug_print_piece_coords(&self){
        for row in self.board.iter() {
            for piece in row.iter() {
                print!(" {} ", piece.coord)
            }
            println!();
        }
    }

    pub fn export_fen(&self) -> String {
        let mut fen = String::new();
    
        for (i, row) in self.board.iter().enumerate() {
            let mut empty_count = 0;
    
            for piece in row.iter() {
                if piece.piece_type == PieceType::None {
                    empty_count += 1; // Increase the empty square count
                }
                else {
                    // Append the count of empty squares if any
                    if empty_count > 0 {
                        fen += &empty_count.to_string();
                        empty_count = 0; // Reset the count after using it
                    }
    
                    // Convert piece type to the FEN character (uppercase for white, lowercase for black)
                    let letter = PieceType::to_char(piece.piece_type, piece.white).to_string();
                    fen += &letter;
                }
            }
    
            // Append any remaining empty squares at the end of the row
            if empty_count > 0 {
                fen += &empty_count.to_string();
            }
    
            // Add '/' separator if it's not the last row
            if i < BOARD_SIZE - 1 {
                fen += "/";
            }
        }
    
        // Add the game status (turn to move)
        if self.white_to_move {
            fen += " w";
        } else {
            fen += " b";
        }
    
        return fen;
    }

    // TODO: Consider robust error handling rather than `panic` usage.
    pub fn import_fen(&mut self, fen: &str) {
        // Trim the string.
        let trimmed_full_fen = fen.trim();

        // Split by first space to separate board position from the rest of the FEN components.
        let mut parts = trimmed_full_fen.split(' ');
        let board_str = parts.next().expect("No board position found in FEN.");

        // Prepare to populate our board.
        let rows = board_str.split('/');
        let mut y_pos = 0;

        // Parse each row of the board.
        for row in rows {

            // For each char in the row, we will either have a character or a number.
            let mut x_pos = 0;
            for c in row.chars() {

                // Handles empty spaces on the board.
                if c.is_digit(10) {
                    let num_empties = c.to_digit(10).expect("Failed to parse digit.") as usize;

                    if num_empties < 1 || num_empties > 8 {
                        panic!("Invalid number of empty spaces: {}", num_empties);
                    }

                    // Mark each square as empty.
                    for _ in 0..num_empties {
                        self.board[y_pos][x_pos].piece_type = PieceType::None;

                        // TODO: Move this into default function. The position of a piece should not depend on the FEN.
                        self.board[y_pos][x_pos].coord.x = x_pos;
                        self.board[y_pos][x_pos].coord.y = y_pos;
                        x_pos += 1;
                    }

                    continue;
                }

                // Place the piece on the board.
                let piece = &mut self.board[y_pos][x_pos];
                piece.white = c.is_ascii_uppercase();
                piece.piece_type = match c.to_ascii_lowercase() {
                    'k' => PieceType::King,
                    'q' => PieceType::Queen,
                    'r' => PieceType::Rook,
                    'b' => PieceType::Bishop,
                    'n' => PieceType::Knight,
                    'p' => PieceType::Pawn,
                    _ => panic!("Unexpaced piece letter {}", c),
                };

                // TODO: Move this into default function. The position of a piece should not depend on the FEN.
                piece.coord.x = x_pos;
                piece.coord.y = y_pos;

                x_pos += 1;
            }

            // Ensure that the board has exactly 8 cols.
            if x_pos != 8 {
                panic!("Board must have exactly 8 columns. We parsed: {}", x_pos);
            }

            y_pos += 1;
        }

        // Ensure that the board has exactly 8 rows.
        if y_pos != 8 {
            panic!("Board must have exactly 8 rows. We parsed: {}", y_pos);
        }

        // Store whose turn it is to move.
        let whose_turn = parts.next().expect("Not sure whose turn it is.");
        if whose_turn.to_ascii_lowercase() == "w" {
            self.white_to_move = true;
        } else if whose_turn.to_ascii_lowercase() == "b" {
            self.white_to_move = false;
        } else {
            panic!("Unexpected character for whose turn it is: {}. Should be 'w' or 'b'.", whose_turn);
        }

        // Castling.
        let castling_rights_str = parts.next();
        match castling_rights_str {
            Some(_s) => {
                // TODO: Implement this.
            },
            None => { return },
        }

        // En-Passant target.
        let en_passant_target_str = parts.next();
        match en_passant_target_str {
            Some(s) => {
                // Try to parse the string as a coordinate.
                let parsed_coord = Coord::str_to_coord(s);
                if parsed_coord.is_ok() {
                    self.en_passant_target = Some(parsed_coord.unwrap());
                }
                else {
                    self.en_passant_target = None;
                }
            },
            None => { return },
        }

        // Half-Moves since last pawn move and capture (for 50 move rule).
        let half_move_str = parts.next();
        match half_move_str {
            Some(s) => {
                let parsed_number: Result<u32, _> = s.parse();
                if parsed_number.is_ok() {
                    self.half_move_count_non_pawn_non_capture = parsed_number.unwrap();
                }
            },
            None => { return }
        }

        // Full move count. Incremented after black moves.
        let full_move_count_str = parts.next();
        match full_move_count_str {
            Some(s) => {
                let parsed_number: Result<u32, _> = s.parse();
                if parsed_number.is_ok() {
                    self.full_move_count = parsed_number.unwrap();
                }
            },
            None => { return }
        }
    }

    pub fn get_all_legal_moves(&self) -> Vec<Move> {
        // Get all pieces to look at.
        let mut legal_moves: Vec<Move> = vec![];
        for row in self.board.iter() {
            for piece in row.iter() {
                if piece.piece_type != PieceType::None && piece.white == self.white_to_move {
                    legal_moves.append(&mut piece.get_legal_moves(self));
                }
            }
        }

        return legal_moves;
    }

    pub fn get_piece_at_coord(&self, coord: &Coord) -> &Piece {
        if coord.x >= BOARD_SIZE || coord.y >= BOARD_SIZE {
            panic!("Attempted to reference an out of bounds location. x:{} y:{}", coord.x, coord.y);
        }

        return &self.board[coord.y][coord.x];
    }

    // TODO: Optimize this? And refactor to use current game state.
    pub fn is_in_check(&self) -> bool {
        // Find our king's coordinates.
        let mut king_coord: Option<Coord> = None;
        for row in self.board.iter() {
            for piece in row.iter() {
                if piece.piece_type == PieceType::King && piece.white == self.white_to_move {
                    king_coord = Some(piece.coord);
                }
            }
        }

        // If we can't find our king, something has gone seriously wrong...
        if king_coord.is_none() {
            panic!("Could not find king on the board, should be impossible.");
        }

        // Check all enemy pieces on the board, see if they are attacking our king.
        for row in self.board.iter() {
            for piece in row.iter() {
                if
                    piece.piece_type != PieceType::None
                    && piece.white != self.white_to_move
                    && piece.is_attacking_coord(&king_coord.unwrap(), self)
                {
                    return true;
                }
            }
        }

        // If we find nothing, we aren't in check.
        return false;
    }

    pub fn does_move_put_self_in_check(&self, m: &Move) -> bool {
        // Make the move on a cloned board, see if it works.
        let mut cloned_game = self.clone();

        // Get the piece that is moving.
        let piece_to_move = cloned_game.board[m.from.y][m.from.x].clone();

        // Make the new square the new piece.
        cloned_game.board[m.to.y][m.to.x] = piece_to_move;

        // Correct the coordinates, and the moved status.
        cloned_game.board[m.to.y][m.to.x].coord = Coord {
            x: m.to.x,
            y: m.to.y,
        };

        // Clear out the old square.
        cloned_game.board[m.from.y][m.from.x].piece_type = PieceType::None;

        return cloned_game.is_in_check();
    }

    pub fn is_in_checkmate(&self) -> bool {
        return self.is_in_check() && self.legal_moves.len() == 0;
    }

    pub fn is_in_stalemate(&self) -> bool {
        return !self.is_in_check() && self.legal_moves.len() == 0;
    }

    // There is something wrong with this function right now...
    pub fn update_game_state(&mut self) {
        if self.is_in_checkmate() {
            if self.white_to_move {
                self.state = GameState::BlackWins;
            }
            else {
                self.state = GameState::WhiteWins;
            }
        }
        else if self.is_in_stalemate() {
            self.state = GameState::Draw;
        }
        else {
            self.state = GameState::InProgress;
        }
    }

    // Does not check if a move is legal. Also updates game state.
    pub fn make_move(&mut self, m: &Move) {
        // Get the piece that is moving.
        let piece_to_move = self.board[m.from.y][m.from.x].clone();

        // Special logic involving casting rights.
        if piece_to_move.piece_type == PieceType::King {
            if piece_to_move.white {
                self.can_white_castle_short = false;
                self.can_white_castle_long = false;
            } else {
                self.can_black_castle_short = false;
                self.can_black_castle_long = false;
            }

            // If the king is moving two squares, we are castling.
            let x_dist =  (m.from.x as isize - m.to.x as isize).abs();
            if x_dist == 2 {
                let castle_direction: CastleSides;
                if m.to.x > m.from.x {
                    castle_direction = CastleSides::Short;
                }
                else {
                    castle_direction = CastleSides::Long;
                }

                // Erase the rook on the starting square.
                let coord_of_rook_to_erase: Coord = CastleSides::get_rook_start_coord(&castle_direction, self.white_to_move);
                self.board[coord_of_rook_to_erase.y][coord_of_rook_to_erase.x].piece_type = PieceType::None;

                let rook_offset: i32 = match castle_direction {
                    CastleSides::Short => { -1 }
                    CastleSides::Long => { 1 }
                };

                // Place the rook on the right side of the king.
                let rook_x_coord = m.to.x as i32 + rook_offset;
                self.board[m.to.y][rook_x_coord as usize].piece_type = PieceType::Rook;
                self.board[m.to.y][rook_x_coord as usize].white = piece_to_move.white;
            }
        }

        // If the rook is leaving it's initial square, we can no longer castle that way.
        // TODO: Clean this up. There has to be a better way of storing this.
        if piece_to_move.piece_type == PieceType::Rook {
            if
                piece_to_move.white
                && self.can_white_castle_short
                && piece_to_move.coord == CastleSides::get_rook_start_coord(&CastleSides::Short, true)
            {
                self.can_white_castle_short = false;
            }
            else if
                piece_to_move.white
                && self.can_white_castle_long
                && piece_to_move.coord == CastleSides::get_rook_start_coord(&CastleSides::Long, true)
            {
                self.can_white_castle_long = false;
            }
            else if
                !piece_to_move.white
                && self.can_black_castle_short
                && piece_to_move.coord == CastleSides::get_rook_start_coord(&CastleSides::Short, false)
            {
                self.can_black_castle_short = false;
            }
            else if
                !piece_to_move.white
                && self.can_black_castle_long
                && piece_to_move.coord == CastleSides::get_rook_start_coord(&CastleSides::Long, false)
            {
                self.can_black_castle_long = false;
            }
        }

        let target_square = &self.board[m.to.y][m.to.x];

        /*
            Special check for en-passant captures. If we are moving a pawn to an empty square and we are capturing,
            this is an en-passant capture. As such, we need to delete the piece at the en-passant square as well.
        */
        if
            piece_to_move.piece_type == PieceType::Pawn
            && target_square.piece_type == PieceType::None
            && m.is_capture == Some(true)
        {
            let y_offset_to_clear: usize = if self.white_to_move { m.to.y + 1 } else { m.to.y - 1 };
            self.board[y_offset_to_clear][m.to.x].piece_type = PieceType::None;
        }

        // Make the new square the new piece.
        self.board[m.to.y][m.to.x] = piece_to_move;

        // Correct the coordinates
        self.board[m.to.y][m.to.x].coord = Coord {
            x: m.to.x,
            y: m.to.y,
        };

        // If we are promoting a pawn, update the piece type!
        if piece_to_move.piece_type == PieceType::Pawn && m.pawn_promoting_to != None {
            self.board[m.to.y][m.to.x].piece_type = m.pawn_promoting_to.unwrap();
        }

        // Clear out the old square.
        self.board[m.from.y][m.from.x].piece_type = PieceType::None;

        // Track en-passant target data.
        self.en_passant_target = m.en_pessant_target_coord;

        if m.is_capture == Some(true) || piece_to_move.piece_type == PieceType::Pawn {
            self.half_move_count_non_pawn_non_capture = 0;
        }
        else {
            self.half_move_count_non_pawn_non_capture += 1;
        }

        // Track full move counter.
        if !self.white_to_move {
            self.full_move_count += 1;
        }

        // Make it the other player's turn.
        self.white_to_move = !self.white_to_move;

        // Update legal moves in the position.
        self.update_legal_moves();

        // TODO: Run this? Causes stack overflows right now...
        self.update_game_state();
    }

    pub fn update_legal_moves(&mut self) {
        self.legal_moves.clear();
        self.legal_moves = self.get_all_legal_moves();
    }

    pub fn print_all_legal_moves(&self) {
        print!("All legal moves: ");
        for m in self.legal_moves.iter() {
            print!("{} ", m);
        }
        println!();
    }

    pub fn print_debug_game_state(&self) {
        print!("Game state: ");
        self.state.print_game_state();

        if self.white_to_move {
            print!("It is white to move.");
        } else {
            print!("It is black to move.");
        }

        println!(" And they have {} legal moves.", self.legal_moves.len());
        print!("En-Passant target square: ");
        match self.en_passant_target {
            Some(t) => println!("{}", t),
            None => println!("None"),
        }

        println!("In check? {}", self.is_in_check());
        println!("Full move count: {}", self.full_move_count);
        println!("Half moves with no capture and pawn move: {}", self.half_move_count_non_pawn_non_capture);
    }

    pub fn play_game_vs_bot(&mut self) {

        
        self.import_fen(INITIAL_GAME_STATE_FEN);
        //self.import_fen("rnb1kbnr/pppp1ppp/11111111/1111p111/1111PP1q/111111P1/PPPP111P/RNBQKBNR b");
        //self.import_fen("rnbqkbnr/pppp1ppp/8/4p3/5PP1/8/PPPPP2P/RNBQKBNR b"); // M1 for black.
        //self.import_fen("rnbqkbnr/p1pppppp/8/1p3P2/8/8/PPPPP1PP/RNBQKBNR b KQkq - 0 2"); // Testing for en-passant.
        //self.import_fen("rnbqkbnr/p1pp1ppp/8/1p2pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3"); // Direct capture allowed.
        //self.import_fen("rnbqkbnr/pppppppp/8/8/4B3/4N3/PPPPPPPP/RNBQK2R w KQkq - 0 1"); // Can castle short?
        //self.import_fen("rnbqkbnr/pppppppp/8/8/1QN1B3/2B1N3/PPPPPPPP/R3K2R w KQkq - 0 1"); // Can castle long and short.
        //self.import_fen("rnbqk3/1pppp1P1/7P/5P2/1QN1B3/1PB1N3/pPPPP3/4K2R w Kq - 0 1"); // Pawn promotion.
        self.update_legal_moves();
        println!("Starting a new game.");

        let mut iter_counter: i32 = 0;
        loop {
            // Print the board.
            self.print_board();

            // See if game is over.
            if self.state != GameState::InProgress {
                //self.state.print_game_state();
                self.print_debug_game_state();
                break;
            }

            // Player move.
            println!("It is your turn. Enter a move.");

            // Read the user input.
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to readline. Not sure what went wrong.");

            // Remove endline.
            input = String::from(input.trim());

            // TEMPORARY DEBUG OUTPUT FEN
            if input == "export" {
                println!("{}", self.export_fen());
                continue;
            }

            // TEMPORARY DEBUG OUTPUT GAME STATE
            if input == "debug" {
                self.print_debug_game_state();
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
            let mut user_move: Option<Move> = None;
            for m in self.legal_moves.iter() {
                if m == &user_move_raw {
                    user_move = Some(m.clone());
                    break;
                }
            }

            match user_move {
                Some(m) => self.make_move(&m),
                None => {
                    println!("That is not one of your legal moves. Try again.");
                    self.print_all_legal_moves();
                    continue;
                }
            }

            println!("Made the move...");

            // See if game is over.
            if self.state != GameState::InProgress {
                self.print_board();
                //self.state.print_game_state();
                self.print_debug_game_state();
                break;
            }

            // TODO: Let the bot make a move.
            self.make_move(
                &self.get_bot_move()
            );

            // Temporary guard for oopsies...
            iter_counter += 1;
            if iter_counter > 1000 {
                panic!("Dev likely did something wrong, hit 1000 iterations.");
            }
        }
    }

    pub fn get_bot_move(&self) -> Move {
        // Get all legal moves.
        let moves: Vec<Move> = self.get_all_legal_moves();

        // Pick a random one.
        let m_option = moves.choose(&mut rand::thread_rng());

        match m_option {
            Some(m) => return m.clone(),
            None => panic!("Something has gone very wrong. Tried to get a bot move when none available."),
        }
    }
}
