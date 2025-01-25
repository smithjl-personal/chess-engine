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
    let _ = lichess::main().await;
}
