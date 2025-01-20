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
    new_game.set_legal_moves(None);
    //new_game.play_game_vs_bot();

    println!("Initial game state:");
    new_game.print_board();


    // Benchmark how long it takes to get a bot move.
    use std::time::Instant;
    let now = Instant::now();


    let bot_move = new_game.get_bot_move();

    let elapsed = now.elapsed();
    println!("Bot thinks we should play {}\n Elapsed: {:.2?}", bot_move.move_to_str(), elapsed);
    //println!("Minimax called {} times", new_game.debug_minimax_calls);

    // new_game.set_legal_moves();
    // new_game.print_board();
    // new_game.print_legal_moves();
    // println!("Evaluation: {}", new_game.evaluate_board());

    // Test new bitboard logic.
    // let str_square = "a4";
    // let square = bitboard::str_coord_to_square(&str_square).unwrap();
    // let is_attacked = new_game.is_square_attacked(square, &bitboard::Color::White);
    // println!("Is {str_square} attacked? {is_attacked}");

    
    // bitboard::print_bitboard(new_game.occupancy_bitboards[2]); // All occupancies
    // bitboard::print_bitboard(new_game.occupancy_bitboards[1]); // Black occupancies
    // bitboard::print_bitboard(new_game.occupancy_bitboards[0]); // White occupancies

    // let from_square = bitboard::str_coord_to_square("g7").unwrap();
    // let to_square = bitboard::str_coord_to_square("g8").unwrap();
    // let user_input_move = bitboard::Move::new(from_square, to_square);
    // let user_input_move = bitboard::Move::str_to_move("g7g8q").expect("Testing, so may be errors...");

    // // Has move with populated meta-data.
    // let this_move = new_game.choose_move_from_legal_move(&user_input_move).expect("Testing, but should be non-null...");

    // new_game.make_move(&this_move, true);
    // new_game.print_board();
    // new_game.print_legal_moves();
    // println!("Evaluation: {}", new_game.evaluate_board());

}
