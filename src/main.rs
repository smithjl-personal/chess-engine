use std::array;
const BOARD_SIZE: usize = 8;

#[derive(PartialEq)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    None,
}

#[derive(Debug)]
struct Coord {
    x: usize,
    y: usize,
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = (b'a' + self.x as u8) as char;
        let rank = BOARD_SIZE - self.y;
        write!(f, "{}{}", file, rank)
    }
}

#[derive(Debug)]
struct Move {
    from: Coord,
    to: Coord,
    // TODO: Consider storing if move is a capture, and if move is a check. Will help find good moves.
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.from, self.to)
    }
}

// Struct for each piece on the board
struct Piece {
    piece_type: PieceType,
    white: bool,
    short_name: String,

    // To see our legal moves, we track the position of each actual piece.
    x: usize,
    y: usize,

    // Important for castling, and pawns.
    has_moved: bool,
}

// Implement the Default trait for Piece
impl Default for Piece {
    fn default() -> Self {
        Piece {
            piece_type: PieceType::None,
            white: false,
            short_name: String::from(""),

            x: 0,
            y: 0,

            has_moved: false,
        }
    }
}

impl Piece {
    fn get_legal_moves(&self, game: &Game) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        // We can only move if it is our turn.
        if self.white != game.white_to_move {
            return moves;
        }

        // TODO: Is there a way to encapsulate this logic insite the PieceType enum somehow?
        match self.piece_type {
            PieceType::King => {
                /*
                 * King can move one tile in any direction. `..=` means range inclusive.
                 * This section handles normal parts, not castling.
                 */
                for y  in (self.y.saturating_sub(1))..=(self.y + 1).min(BOARD_SIZE - 1) {
                    for x in (self.x.saturating_sub(1))..=(self.x + 1).min(BOARD_SIZE - 1) {
                        let target_piece: &Piece = &game.board[y as usize][x as usize];

                        // We cannot move on a piece that is our own color.
                        if target_piece.piece_type != PieceType::None && target_piece.white == self.white {
                            continue;
                        }

                        // We cannot make a move that would put us in check.
                        // TODO: Implement this?

                        // Otherwise, move is legal.
                        let from_tile: Coord = Coord {
                            x: self.x,
                            y: self.y,
                        };
                        let to_tile: Coord = Coord {
                            x: x,
                            y: y,
                        };
                        moves.push(Move {
                            from: from_tile,
                            to: to_tile,
                        });
                    }
                }

                // TODO: Check for castling.
            }
            PieceType::Queen => {}
            PieceType::Rook => {}
            PieceType::Bishop => {}
            PieceType::Knight => {}
            PieceType::Pawn => {}
            PieceType::None => {
                return moves;
            }
        }

        return moves;
    }
}

// Struct for the chessboard, which is a 2D array of Pieces
struct Game {
    board: [[Piece; BOARD_SIZE]; BOARD_SIZE],
    white_to_move: bool,
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
    fn print_board(&self) {
        // Here we can print the board or something relevant to it
        for row in self.board.iter() {
            for piece in row.iter() {
                if piece.piece_type == PieceType::None {
                    print!(" E "); // E for empty piece
                } else {
                    print!(" {} ", piece.short_name); // Print the short name of the piece
                }
            }
            println!(); // Print new line after each row
        }
    }

    fn import_fen(&mut self, fen: &str) {
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

                piece.x = x_pos;
                piece.y = y_pos;

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

    fn print_all_legal_moves(&self) {
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
}

fn main() {
    let mut game = Game::default();
    //game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR b KQkq - 0 1");
    game.print_board();
    game.print_all_legal_moves();
}