use std::array;
use crate::{piece::Piece, my_move::Move, piece_type::PieceType, coord::Coord, constants::BOARD_SIZE};

// Struct for the chessboard, which is a 2D array of Pieces
#[derive(Clone)]
pub struct Game {
    pub board: [[Piece; BOARD_SIZE]; BOARD_SIZE],
    pub white_to_move: bool,
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

    pub fn print_all_legal_moves(&self) {
        println!("Called print all legal moves!");

        // Get all pieces to look at.
        let mut our_pieces: Vec<&Piece> = vec![];
        for row in self.board.iter() {
            for piece in row.iter() {
                if piece.piece_type != PieceType::None && piece.white == self.white_to_move {
                    our_pieces.push(&piece);
                }
            }
        }

        /*
            There is likely a better way of doing this. But just getting something on the screen.

            To figure out what legal moves a piece has, we need the following
            1. Which color the piece is.
            2. The piece type.
            3. Are we in check?
            4. Is the piece pinned?
            5. Castling rules?
            6. En-Pessant rules?
        */
        for piece in our_pieces.iter() {
            print!("Looking at moves for {}: ", piece.short_name);
            let moves: Vec<Move> = piece.get_legal_moves(self);
            for m in moves.iter() {
                //print!("{:#?}", m); // debug pretty
                //print!("{:?}", m); // debug
                print!("{} ", m); // normal
            }
            println!();
        }

    }


    // TODO: Implement this.
    pub fn is_in_check(&self, _check_white: bool) -> bool {
        return false;
    }

    // Does not check if a move is legal.
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
    }
}
