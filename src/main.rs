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
    let _ = new_game.import_fen("r3k2r/5n1P/1N1Bpp2/1pPR3N/1pP1Q3/1P1b1P1p/P5PP/R3K2R b KQkq - 0 1");
    new_game.print_board();
    new_game.print_legal_moves();
}
