use crate::constants::{BOARD_SIZE, KNIGHT_MOVES};  // Importing the constant
use crate::{piece_type::PieceType, coord::Coord, g::Game, my_move::Move};


// Struct for each piece on the board
#[derive(Clone, Debug)]
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
                moves.push(Move { from: from_tile, to: to_tile, is_capture: false, });
            }

            // If the tile contains a friendly piece, stop in that direction.
            else if game.board[y_usize][x_usize].white == self.white {
                break;
            }

            // If the tile contains an enemy piece, store the move and stop.
            else {
                let from_tile = self.coord;
                let to_tile = Coord { x: x_usize, y: y_usize };
                moves.push(Move { from: from_tile, to: to_tile, is_capture: true, });
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

                        // If square is open, we can do the move.
                        if target_piece.piece_type == PieceType::None {
                            moves.push(Move {
                                from: self.coord,
                                to: target_piece.coord,
                                is_capture: false,
                            });
                        }

                        // We cannot move on a piece that is our own color.
                        else if target_piece.white == self.white {
                            continue;
                        }

                        // Last case, move must be a capture.
                        else {
                            moves.push(Move {
                                from: self.coord,
                                to: target_piece.coord,
                                is_capture: true,
                            });
                        }
                    }
                }

                // TODO: Check for castling.
                if !self.has_moved {}
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
                            is_capture: false,
                        });
                    }

                    // If square is enemy, track it!
                    else if target_square.white != self.white {
                        moves.push(Move {
                            from: self.coord,
                            to: target_square.coord,
                            is_capture: true,
                        });
                    }
                }
            }
            PieceType::Pawn => {
                // White pawns go up the board, black ones go down.
                let direction: i32 = if self.white {-1} else {1};

                // We do not need to do any bounds checking for normal moves because of how pawn moves work.
                // If a pawn makes it to the top/bottom rank, it promotes and is no longer a pawn.
                let y_pos: i32 = self.coord.y as i32 + direction;

                // Check moves, not captures here.
                let target = &game.board[y_pos as usize][self.coord.x];
                if target.piece_type == PieceType::None {

                    // TODO: Handle pawn promotion.
                    moves.push(Move {
                        from: self.coord,
                        to: target.coord,
                        is_capture: false,
                    });

                    // Check for double square move if we haven't moved yet.
                    if !self.has_moved {
                        let extra_target = &game.board[(y_pos + direction) as usize][self.coord.x];
                        if extra_target.piece_type == PieceType::None {
                            moves.push(Move {
                                from: self.coord,
                                to: extra_target.coord,
                                is_capture: false,
                            });
                        }
                    }
                }

                // TODO: Now check captures. We do need bounds checks here.
            }
            PieceType::None => {
                return moves;
            }
        }

        // TODO: Remove any move that results in us being in check. That would be illegal...
        /*
         * Idea for next time.
         * 1. Add function to `game` that determines if player is in check.
         * 2. Make a new copy of the game, after that move is made.
         * 3. See if we are in check!
         * 
         * We can also use this method to track if a move is a check or not.
         */

        // Try out every move. See if it's illegal.
        let mut moves_final: Vec<Move> = vec![];
        for m in moves.iter() {
            // TODO: Refactor this. Most of this should not need to be in the loop, with good code.

            // Make a duplicate of this game object.
            let mut game_copy = game.clone();

            game_copy.make_move(m);
            if !game_copy.is_in_check(game.white_to_move) {
                moves_final.push(m.clone());
            }
        }

        return moves_final;
    }

    pub fn is_attacking_coord(&self, coord: &Coord, game: &Game) -> bool {
        match self.piece_type {
            PieceType::King => {
                let diff_x: isize = self.coord.x as isize - coord.x as isize;
                let diff_y: isize = self.coord.y as isize - coord.y as isize;

                let x_in_range: bool = -1 <= diff_x && diff_x <= 1;
                let y_in_range: bool = -1 <= diff_y && diff_y <= 1;

                return x_in_range && y_in_range;
            }
            PieceType::Queen => {
                // TODO: Implement this.
                return false;
            }
            PieceType::Rook => {
                let mut dx = 0;
                let mut dy = 0;
                if self.coord.x == coord.x {
                    dx = 0;
                    if self.coord.y > coord.y {
                        dy = -1;
                    }
                    else {
                        dy = 1;
                    }
                }

                else if self.coord.y == coord.y {
                    dy = 0;
                    if self.coord.x > coord.x {
                        dx = -1;
                    }
                    else {
                        dx = 1;
                    }
                }

                // If we don't share the rank or file, we aren't attacking the square.
                else { return false; }

                // println!("Checking dx {} dy {}", dx, dy);

                // TODO: Extract the below logic into a function that can be used by the queen, rook, and bishop.

                // Now try and move in the direction until we reach the square.
                let mut check_x_pos = self.coord.x as i32;
                let mut check_y_pos = self.coord.y as i32;
                loop {

                    //println!("Beginning loop.");

                    check_x_pos += dx;
                    check_y_pos += dy;

                    //println!("Looking at coord x {} y {}", check_x_pos, check_y_pos);

                    // Perform a bounds check.
                    if check_x_pos < 0 || check_y_pos < 0 || check_x_pos >= BOARD_SIZE as i32 || check_y_pos >= BOARD_SIZE as i32{
                        return false;
                    }

                    // println!("We are in bounds.");

                    // If we are on the square, we are done.
                    if check_x_pos as usize == coord.x && check_y_pos as usize == coord.y {
                        // println!("We are reached the target.");
                        return true;
                    }

                    // We need more info to see if we can continue.
                    let piece = &game.board[check_y_pos as usize][check_x_pos as usize];

                    // println!("{:?}", piece);

                    // If square is empty, we can continue.
                    if piece.piece_type == PieceType::None {
                        // println!("Square is empty, keep going.");
                        continue;
                    }

                    // We cannot move over our pieces (normally).
                    else if piece.white == self.white {
                        //println!("Square is our piece, stop.");
                        return false;
                    }

                    // At this point, we must be on an enemy piece that is not the target square. So we stop here.
                    else {
                        return false;
                    }
                }
            }
            PieceType::Bishop => {
                // TODO: Implement this.
                return false;
                
            }
            PieceType::Knight => {
                let diff_x_abs: isize = (self.coord.x as isize - coord.x as isize).abs();
                let diff_y_abs: isize = (self.coord.y as isize - coord.y as isize).abs();

                // Since we take the absolute value, we only need to check positives.
                let valid_a: bool = diff_x_abs == 1 && diff_y_abs == 2;
                let valid_b: bool = diff_y_abs == 1 && diff_x_abs == 2;

                return valid_a || valid_b;
            }
            PieceType::Pawn => {
                let dir: isize = if self.white { -1 } else { 1 };
                let target_y: isize = self.coord.y as isize + dir;

                // Check the y position.
                if target_y != coord.y as isize {
                    return false;
                }

                // Check the left and right x positions.
                let target_left: isize = self.coord.x as isize - 1;
                let target_right: isize = self.coord.x as isize + 1;

                return target_left == coord.x as isize || target_right == coord.x as isize;
                
            }
            PieceType::None => { return false; }
        }
    }
}