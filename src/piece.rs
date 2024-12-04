use crate::constants::{BOARD_SIZE, KNIGHT_MOVES};  // Importing the constant
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
    pub fn generate_moves_in_direction(
        &self, 
        game: &Game, 
        moves: &mut Vec<Move>, 
        dx: i32, 
        dy: i32
    ) {
        let mut x = self.coord.x as i32;
        let mut y = self.coord.y as i32;
    
        loop {
            x += dx;
            y += dy;
    
            // Check if the new position is out of bounds.
            if x < 0 || x >= BOARD_SIZE as i32 || y < 0 || y >= BOARD_SIZE as i32 {
                break;
            }
    
            let x_usize = x as usize;
            let y_usize = y as usize;

            // If the tile is free, store it as a valid move.
            if game.board[y_usize][x_usize].piece_type == PieceType::None {
                let from_tile = self.coord;
                let to_tile = Coord { x: x_usize, y: y_usize };
                moves.push(Move { from: from_tile, to: to_tile });
            }

            // If the tile contains a friendly piece, stop in that direction.
            else if game.board[y_usize][x_usize].white == self.white {
                break;
            }

            // If the tile contains an enemy piece, store the move and stop.
            else {
                let from_tile = self.coord;
                let to_tile = Coord { x: x_usize, y: y_usize };
                moves.push(Move { from: from_tile, to: to_tile });
                break;
            }
        }
    }

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
            PieceType::Queen => {
                // Right
                self.generate_moves_in_direction(game, &mut moves, 1, 0);

                // Left
                self.generate_moves_in_direction(game, &mut moves, -1, 0);

                // Up
                self.generate_moves_in_direction(game, &mut moves, 0, -1);

                // Down
                self.generate_moves_in_direction(game, &mut moves, 0, 1);

                // Top-right
                self.generate_moves_in_direction(game, &mut moves, 1, -1);

                // Top-left
                self.generate_moves_in_direction(game, &mut moves, -1, -1);

                // Bottom-left
                self.generate_moves_in_direction(game, &mut moves, -1, 1);

                // Bottom-right
                self.generate_moves_in_direction(game, &mut moves, 1, 1);
            }
            PieceType::Rook => {
                // Right
                self.generate_moves_in_direction(game, &mut moves, 1, 0);

                // Left
                self.generate_moves_in_direction(game, &mut moves, -1, 0);

                // Up
                self.generate_moves_in_direction(game, &mut moves, 0, -1);

                // Down
                self.generate_moves_in_direction(game, &mut moves, 0, 1);
            }
            PieceType::Bishop => {
                // Top-right
                self.generate_moves_in_direction(game, &mut moves, 1, -1);

                // Top-left
                self.generate_moves_in_direction(game, &mut moves, -1, -1);

                // Bottom-left
                self.generate_moves_in_direction(game, &mut moves, -1, 1);

                // Bottom-right
                self.generate_moves_in_direction(game, &mut moves, 1, 1);
            }
            PieceType::Knight => {

                // For each possible knight move.
                for (y, x) in KNIGHT_MOVES.iter(){

                    // Store the offset.
                    let y_pos = y + self.coord.y as i16;
                    let x_pos = x + self.coord.x as i16;

                    // Skip if out of bounds.
                    if x_pos < 0 || x_pos >= BOARD_SIZE as i16 || y_pos < 0 || y_pos >= BOARD_SIZE as i16 {
                        continue;
                    }

                    // Store our target location.
                    let target_square = &game.board[y_pos as usize][x_pos as usize];

                    // If square is empty, track it!
                    if target_square.piece_type == PieceType::None {
                        moves.push(Move {
                            from: self.coord,
                            to: target_square.coord,
                        });
                    }

                    // If square is enemy, track it!
                    else if target_square.white != self.white {
                        moves.push(Move {
                            from: self.coord,
                            to: target_square.coord,
                        });
                    }
                }
            }
            PieceType::Pawn => {}
            PieceType::None => {
                return moves;
            }
        }

        return moves;
    }
}