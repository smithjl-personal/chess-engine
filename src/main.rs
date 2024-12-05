pub mod piece_type;
pub mod coord;
pub mod piece;
pub mod my_move;
pub mod g; // Something is wrong with rust-analyzer. This is the only way it will pick up the changes right now.
pub mod constants;
pub mod tests;

use crate::g::Game;

fn main() {
    let mut game: Game = g::Game::default();
    game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR b KQkq - 0 1");
    game.print_board();
    game.print_all_legal_moves();
}