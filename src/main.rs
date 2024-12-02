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

// Struct for each piece on the board
struct Piece {
    piece_type: PieceType,
    white: bool,
    short_name: String,
}

// Implement the Default trait for Piece
impl Default for Piece {
    fn default() -> Self {
        Piece {
            piece_type: PieceType::None,
            white: false,
            short_name: String::from(""),
        }
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

fn main() {
    // Example starting postion.
    // rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    let game = import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // Print the board.
    // TODO: Consider making this a part of the game struct?
    print_board(&game);
}

fn print_board(game: &Game) {
    // Here we can print the board or something relevant to it
    for row in game.board.iter() {
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

fn import_fen(fen: &str) -> Game {
    // Trim the string.
    let trimmed_full_fen = fen.trim();

    // Split by first space to separate board position from the rest of the FEN components.
    let mut parts = trimmed_full_fen.split(' ');
    let board_str = parts.next().expect("No board position found in FEN.");

    // Prepare to populate our board.
    let rows = board_str.split('/');
    let valid_piece_letters = ['r', 'n', 'b', 'q', 'k', 'p'];
    let mut game: Game = Game::default();
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

                // Board is initially empty, so just skip the number of times indicated.
                x_pos += num_empties;
                continue;
            }

            let piece_letter = c.to_ascii_lowercase();
            if !valid_piece_letters.contains(&piece_letter) {
                panic!("Unexpected piece symbol found: {}", c);
            }
            

            // Place the piece on the board.
            let piece = &mut game.board[y_pos][x_pos];
            piece.piece_type = piece_char_to_type(&piece_letter);
            piece.white = c.is_ascii_uppercase();
            piece.short_name = c.to_string();

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
        game.white_to_move = true;
    } else if whose_turn.to_ascii_lowercase() == "b" {
        game.white_to_move = false;
    } else {
        panic!("Unexpected character for whose turn it is: {}. Should be 'w' or 'b'.", whose_turn);
    }

    return game;
}

/*
    This code assumes the piece name is valid. Consider consolidating this block somehow with the import.
*/
fn piece_char_to_type(c: &char) -> PieceType {

    // TODO: Can this be cleaned up with a `match` statement?
    if *c == 'k' {
        return PieceType::King;
    }
    else if *c == 'q' {
        return PieceType::Queen;
    }
    else if *c == 'r' {
        return PieceType::Rook;
    }
    else if *c == 'b' {
        return PieceType::Bishop;
    }
    else if *c == 'n' {
        return PieceType::Knight;
    }
    else if *c == 'p' {
        return PieceType::Pawn;
    }
    else {
        panic!("Unknown piece name given, {}", c);
    }
}