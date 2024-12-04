mod piece_type;
mod coord;
mod piece;
mod my_move;
mod game;

use crate::game::Game;
//use crate::piece::Piece;

fn main() {
    let mut game: Game = Game::default();
    //game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR b KQkq - 0 1");
    game.print_board();
    game.print_all_legal_moves();
}