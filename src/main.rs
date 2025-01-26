pub mod bitboard;
pub mod constants;
pub mod lichess;
pub mod lichess_structs;
pub mod runtime_calculated_constants;
pub mod transposition_table_entry;
pub mod r#move;
pub mod piece_type;
pub mod color;
pub mod castle_sides;
pub mod helpers;

#[tokio::main]
async fn main() {
    // Lichess bot.
    let _ = lichess::main().await;

    // Testing iterative deepening.
    // let c = runtime_calculated_constants::Constants::new();
    // let mut new_game = bitboard::ChessGame::new(&c);
    // let _ = new_game.import_fen("r2qk2r/5nPP/3Bpp2/1pPR3N/1pP1Q3/1P1b1P1p/P5PP/R3K2R w KQkq b6 0 1");
    // new_game.set_legal_moves(None);

    // println!("Initial game state:");
    // new_game.print_board();

    // println!("Try iterative deepening.");
    // let (best_eval, best_move) = new_game.iterative_deepening_minimax();

    // println!("The best move we found was {} at eval {}.", best_move.unwrap().move_to_str(), best_eval);

    // new_game.print_debug_game_state_str();
}
