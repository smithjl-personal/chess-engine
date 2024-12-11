use crate::piece_type::PieceType;

pub const BOARD_SIZE: usize = 8;
pub const KNIGHT_MOVES: [(i16, i16); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, 2),
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
];
pub const INITIAL_GAME_STATE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const PAWN_PROMOTION_CHOICES: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Knight,
    PieceType::Bishop,
];
