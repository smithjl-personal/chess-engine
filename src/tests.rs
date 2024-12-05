
// wtf is going on here with 'self'?
use crate::{constants::BOARD_SIZE, coord::Coord, g::{self}};

pub fn run_all_tests(){
    // Do nothing right now!
    // let mut game: Game = g::Game::default();
    let mut game = g::Game::default();
    game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    game.print_board();
    //game.print_all_legal_moves();

    // Testing targeting functions.
    //let piece = &game.board[6][1]; // Pawn on b2
    //let piece = &game.board[7][1]; // Knight on b1
    let piece = &game.board[7][4]; // King on e1
    //let coord = str_tile_to_coord("a3");
    //let coord = str_tile_to_coord("b3");
    //let coord = str_tile_to_coord("c3");
    let coord = str_tile_to_coord("f1");


    let attacking = piece.is_attacking_coord(&coord);
    println!("Tile {} is attacking {}: {}", piece.coord, coord, attacking);
}

// Helper. Should this be in the coord file?
fn str_tile_to_coord(s: &str) -> Coord {
    if s.len() != 2 {
        panic!("String {} is not a valid coordinate. Not length 2.", s);
    }

    // We know the length is 2, so we can safely unwrap here.
    //let mut ch = s.chars();
    let file_letter = s.chars().nth(0).unwrap().to_ascii_lowercase();
    let rank_number = s.chars().nth(1).unwrap();

    // Attempt conversion for file letter.
    let x: i32 =  file_letter as i32 - 'a' as i32;
    if x < 0 || x >= BOARD_SIZE as i32 {
        panic!("Invalid file letter: {}", file_letter);
    }

    let y: i32 = BOARD_SIZE as i32 - rank_number.to_digit(10).expect("Cannot convert rank character to a digit.") as i32;
    if y < 0 || y >= BOARD_SIZE as i32 {
        panic!("Digit referenced `{}` is outside board size {}.", rank_number, BOARD_SIZE);
    }

    return Coord {
        x: x as usize,
        y: y as usize,
    }
}