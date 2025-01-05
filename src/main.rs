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
    let _ = new_game.import_fen("k5r1/5n1P/3ppp2/1pPR4/1pP1Q3/1P3P1p/P5PP/K7 b - c3 0 1");
    new_game.print_board();
    new_game.print_legal_moves();
}
