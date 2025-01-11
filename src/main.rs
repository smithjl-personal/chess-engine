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

    // let this_move = bitboard::Move {
    //     from: bitboard::str_coord_to_square("e1").unwrap(),
    //     to: bitboard::str_coord_to_square("c1").unwrap(),
    //     is_capture: Some(false),
    //     is_check: Some(false),
    //     next_en_pessant_target_coord: None,
    //     pawn_promoting_to: None,
    //     castle_side: Some(bitboard::CastleSides::Long),
    // };

    //new_game.make_move(&this_move, false);
    //new_game.print_board();
    // print_bitboard(new_game.all_occupancies);
    //new_game.print_legal_moves();
}
