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
    let _ = lichess::main().await;
    //let _ = tests::test_performance_of_minimax();
}
