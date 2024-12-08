
// wtf is going on here with 'self'?
use crate::{constants::BOARD_SIZE, coord::Coord, g::{self}};

pub fn run_all_tests(){
    // test_bishop_attacks();
    // test_queen_attacks();
    // test_is_in_check();
    // test_legal_moves();
    test_game_vs_bot();
}

pub fn test_bishop_attacks() {
    let mut game = g::Game::default();
    game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
    game.print_board();

    let source_coord = str_tile_to_coord("c1");
    let source_piece = game.get_piece_at_coord(&source_coord);
    let str_coords_to_check = vec!["f4", "b2", "c2", "e3"];

    for str_coord in str_coords_to_check.iter() {
        let target_coord = str_tile_to_coord(&str_coord);
        let attacking = source_piece.is_attacking_coord(&target_coord, &game);
        println!("Tile {} is attacking {}: {}", source_piece.coord, target_coord, attacking);
    }
}

pub fn test_queen_attacks() {
    let mut game = g::Game::default();
    game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP2PPP/RNBQKBNR w KQkq - 0 1");
    game.print_board();

    let source_coord = str_tile_to_coord("d1");
    let source_piece = game.get_piece_at_coord(&source_coord);
    let str_coords_to_check = vec!["d8", "d7", "d2", "c2", "e3"];

    for str_coord in str_coords_to_check.iter() {
        let target_coord = str_tile_to_coord(&str_coord);
        let attacking = source_piece.is_attacking_coord(&target_coord, &game);
        println!("Tile {} is attacking {}: {}", source_piece.coord, target_coord, attacking);
    }
}

pub fn test_is_in_check() {
    let mut game = g::Game::default();
    // let look_at_white = false;
    // game.import_fen("rnbqkbnr/ppp2ppp/8/8/8/8/PPP1QPPP/RNB1KBNR w KQkq - 0 1"); // expect: true;
    // let look_at_white = false;
    // game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP1QPPP/RNB1KBNR w KQkq - 0 1"); // expect: false;
    let look_at_white = true;
    game.import_fen("rn1qkbnr/pppppppp/8/b7/8/8/PPP1QPPP/RNB1KBNR w KQkq - 0 1"); // expect: false;
    game.print_board();

    let result = game.is_in_check();
    if look_at_white {
        println!("Is white in check? {}", result);
    } else {
        println!("Is black in check? {}", result);
    }
}

pub fn test_legal_moves() {
    let mut game = g::Game::default();
    game.import_fen("rn1qkbnr/pppppppp/8/b7/8/8/PPP1QPPP/RNB1KBNR w KQkq - 0 1"); // expect: false;
    game.print_board();
    let legal_moves = game.get_all_legal_moves();

    if game.white_to_move {
        print!("It is white to move. ");
    } else {
        print!("It is black to move. ");
    }
    println!("Legal moves: ");
    for m in legal_moves.iter() {
        print!("{} ", m);
    }
}

pub fn test_game_vs_bot() {
    let mut game = g::Game::default();
    game.play_game_vs_bot();
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