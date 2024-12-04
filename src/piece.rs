use crate::constants::BOARD_SIZE;  // Importing the constant
use crate::{piece_type::PieceType, coord::Coord, g::Game, my_move::Move};


// Struct for each piece on the board
pub struct Piece {
    pub piece_type: PieceType,
    pub white: bool,
    pub short_name: String,

    // To see our legal moves, we track the position of each actual piece.
    pub coord: Coord,

    // Important for castling, and pawns.
    pub has_moved: bool,
}

// Implement the Default trait for Piece
impl Default for Piece {
    fn default() -> Self {
        Piece {
            piece_type: PieceType::None,
            white: false,
            short_name: String::from(""),

            coord: Coord {
                x: 0,
                y: 0,
            },

            has_moved: false,
        }
    }
}

impl Piece {
    pub fn get_legal_moves(&self, game: &Game) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        // We can only move if it is our turn.
        if self.white != game.white_to_move {
            return moves;
        }

        // TODO: Is there a way to encapsulate this logic insite the PieceType enum somehow?
        match self.piece_type {
            PieceType::King => {
                /*
                 * King can move one tile in any direction. `..=` means range inclusive.
                 * This section handles normal parts, not castling.
                 */
                for y  in (self.coord.y.saturating_sub(1))..=(self.coord.y + 1).min(BOARD_SIZE - 1) {
                    for x in (self.coord.x.saturating_sub(1))..=(self.coord.x + 1).min(BOARD_SIZE - 1) {
                        let target_piece: &Piece = &game.board[y as usize][x as usize];

                        // We cannot move on a piece that is our own color.
                        if target_piece.piece_type != PieceType::None && target_piece.white == self.white {
                            continue;
                        }

                        // We cannot make a move that would put us in check.
                        // TODO: Implement this?

                        // Otherwise, move is legal.
                        let from_tile: Coord = Coord {
                            x: self.coord.x,
                            y: self.coord.y,
                        };
                        let to_tile: Coord = Coord {
                            x: x,
                            y: y,
                        };
                        moves.push(Move {
                            from: from_tile,
                            to: to_tile,
                        });
                    }
                }

                // TODO: Check for castling.
            }
            PieceType::Queen => {}
            PieceType::Rook => {}
            PieceType::Bishop => {}
            PieceType::Knight => {}
            PieceType::Pawn => {}
            PieceType::None => {
                return moves;
            }
        }

        return moves;
    }
}