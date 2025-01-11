pub mod bitboard;
pub mod castle_sides;
pub mod constants;
pub mod coord;
pub mod g; // Something is wrong with rust-analyzer. This is the only way it will pick up the changes right now.
pub mod game_states;
pub mod lichess;
pub mod lichess_structs;
pub mod my_move;
pub mod piece;
pub mod piece_type;
pub mod tests;

#[tokio::main]
async fn main() {
    //let _ = lichess::main().await;
    //let _ = tests::test_performance_of_minimax();

    let c = bitboard::Constants::new();
    let mut new_game = bitboard::ChessGame::new(&c);
    //let _ = new_game.import_fen(constants::INITIAL_GAME_STATE_FEN);
    let _ = new_game.import_fen("r2qk2r/5nPP/3Bpp2/1pPR3N/1pP1Q3/1P1b1P1p/P5PP/R3K2R w KQkq b6 0 1");
    new_game.print_board();
    new_game.print_legal_moves();

    // Test new bitboard logic.
    // let str_square = "a4";
    // let square = bitboard::str_coord_to_square(&str_square).unwrap();
    // let is_attacked = new_game.is_square_attacked(square, &bitboard::Color::White);
    // println!("Is {str_square} attacked? {is_attacked}");

    
    // bitboard::print_bitboard(new_game.occupancy_bitboards[2]); // All occupancies
    // bitboard::print_bitboard(new_game.occupancy_bitboards[1]); // Black occupancies
    // bitboard::print_bitboard(new_game.occupancy_bitboards[0]); // White occupancies

    let from_square = bitboard::str_coord_to_square("e4").unwrap();
    let to_square = bitboard::str_coord_to_square("e6").unwrap();
    let user_input_move = bitboard::Move::new(from_square, to_square);

    // Has move with populated meta-data.
    let this_move = new_game.choose_move_from_legal_move(&user_input_move).expect("Testing, but should be non-null...");

    new_game.make_move(&this_move, false);
    new_game.print_board();
    new_game.print_legal_moves();
}
