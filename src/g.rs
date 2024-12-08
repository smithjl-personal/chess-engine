use std::{array, io, panic};
use crate::{
    constants::{BOARD_SIZE, INITIAL_GAME_STATE_FEN}
    , coord::Coord
    , my_move::Move
    , piece::Piece
    , piece_type::PieceType
    , game_states::GameState
};

// Struct for the chessboard, which is a 2D array of Pieces
#[derive(Clone)]
pub struct Game {
    pub board: [[Piece; BOARD_SIZE]; BOARD_SIZE],
    pub white_to_move: bool,
    pub state: GameState,
    pub legal_moves: Vec<Move>,
    // TODO: Store castling rights?
    // TODO: Store 'en-pessant' state?
    // TODO: Store half-moves since last pawn capture or advance.
    // TODO: Store full-move count (increment after black moves).
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
                let display: &str = match piece.piece_type {
                    PieceType::None => " ",
                    _=> &piece.short_name,
                };

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
                    let mut letter = PieceType::to_char(piece.piece_type).to_string();
                    if piece.white {
                        letter = letter.to_ascii_uppercase();
                    }
    
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
                piece.short_name = c.to_string();
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
        // println!("Trying to make move, see if we are in check.");

        // Make the move on a cloned board, see if it works.
        let mut cloned_game = self.clone();

        // Get the piece that is moving.
        let piece_to_move = cloned_game.board[m.from.y][m.from.x].clone();

        // Make the new square the new piece.
        cloned_game.board[m.to.y][m.to.x] = piece_to_move;

        // Correct the coordinates, and the moved status.
        cloned_game.board[m.to.y][m.to.x].has_moved = true;
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

        // Make the new square the new piece.
        self.board[m.to.y][m.to.x] = piece_to_move;

        // Correct the coordinates, and the moved status.
        self.board[m.to.y][m.to.x].has_moved = true;
        self.board[m.to.y][m.to.x].coord = Coord {
            x: m.to.x,
            y: m.to.y,
        };

        // Clear out the old square.
        self.board[m.from.y][m.from.x].piece_type = PieceType::None;

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

        println!("In check? {}", self.is_in_check())
    }

    pub fn play_game_vs_bot(&mut self) {

        
        //self.import_fen(INITIAL_GAME_STATE_FEN);
        //self.import_fen("rnb1kbnr/pppp1ppp/11111111/1111p111/1111PP1q/111111P1/PPPP111P/RNBQKBNR b");
        self.import_fen("rnbqkbnr/pppp1ppp/8/4p3/5PP1/8/PPPPP2P/RNBQKBNR b"); // M1 for black.
        self.update_legal_moves();
        println!("Starting a new game. You are white.");

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

            let user_move = match Move::str_to_move(&input) {
                Ok(m) => m,
                Err(msg) => {
                    println!("{}", msg);
                    continue;
                }
            };

            // See if this is in one of the player's legal moves.
            if !self.legal_moves.contains(&user_move) {
                println!("That is not one of your legal moves. Try again.");
                self.print_all_legal_moves();
                continue;
            }

            // Make the move!
            self.make_move(&user_move);

            println!("Made the move...");

            // See if game is over.
            if self.state != GameState::InProgress {
                self.print_board();
                //self.state.print_game_state();
                self.print_debug_game_state();
                break;
            }

            // TODO: Let the bot make a move.

            // Temporary guard for oopsies...
            iter_counter += 1;
            if iter_counter > 1000 {
                panic!("Dev likely did something wrong, hit 1000 iterations.");
            }
        }
    }
}
