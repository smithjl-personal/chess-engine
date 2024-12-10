use crate::constants::{BOARD_SIZE, KNIGHT_MOVES, PAWN_PROMOTION_CHOICES};  // Importing the constant
use crate::{piece_type::PieceType, coord::Coord, g::Game, my_move::Move, castle_sides::CastleSides};


// Struct for each piece on the board
#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub white: bool,

    // To see our legal moves, we track the position of each actual piece.
    pub coord: Coord,
}

// Implement the Default trait for Piece
impl Default for Piece {
    fn default() -> Self {
        Piece {
            piece_type: PieceType::None,
            white: false,

            coord: Coord {
                x: 0,
                y: 0,
            },
        }
    }
}

impl Piece {
    // Should not be used on pawns, does not account for promotion.
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
                moves.push(Move {
                    from: from_tile,
                    to: to_tile,
                    is_capture: Some(false),
                    en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                });
            }

            // If the tile contains a friendly piece, stop in that direction.
            else if game.board[y_usize][x_usize].white == self.white {
                break;
            }

            // If the tile contains an enemy piece, store the move and stop.
            else {
                let from_tile = self.coord;
                let to_tile = Coord { x: x_usize, y: y_usize };
                moves.push(Move {
                    from: from_tile,
                    to: to_tile,
                    is_capture: Some(true),
                    en_pessant_target_coord: None,
                    pawn_promoting_to: None,
                });
                break;
            }
        }
    }

    pub fn can_attack_coord_in_direction(&self, game: &Game, coord: &Coord, dx: i32, dy: i32) -> bool {
        let mut x = self.coord.x as i32;
        let mut y = self.coord.y as i32;

        loop {
            x += dx;
            y += dy;

            // Perform a bounds check.
            if x < 0 || y < 0 || x >= BOARD_SIZE as i32 || y >= BOARD_SIZE as i32 {
                return false;
            }

            if x as usize == coord.x && y as usize == coord.y {
                return true;
            }

            // We need more info to see if we can continue.
            let piece: &Piece = &game.board[y as usize][x as usize];

            // If square is empty, we can continue.
            if piece.piece_type == PieceType::None {
                continue;
            }

            // We cannot move over our pieces (normally).
            else if piece.white == self.white {
                return false;
            }

            // At this point, we must be on an enemy piece that is not the target square. So we stop here.
            else {
                return false;
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
                                is_capture: Some(false),
                                en_pessant_target_coord: None,
                                pawn_promoting_to: None,
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
                                is_capture: Some(true),
                                en_pessant_target_coord: None,
                                pawn_promoting_to: None,
                            });
                        }
                    }
                }

                if self.can_castle(game, CastleSides::Short) {
                    moves.push(Move {
                        from: self.coord,
                        to: Coord {
                            x: self.coord.x + 2,
                            y: self.coord.y,
                        },
                        is_capture: Some(false),
                        en_pessant_target_coord: None,
                        pawn_promoting_to: None,
                    });
                }
                if self.can_castle(game, CastleSides::Long) {
                    moves.push(Move {
                        from: self.coord,
                        to: Coord {
                            x: (self.coord.x - 2) as usize,
                            y: self.coord.y,
                        },
                        is_capture: Some(false),
                        en_pessant_target_coord: None,
                        pawn_promoting_to: None,
                    });
                }
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
                            is_capture: Some(false),
                            en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                        });
                    }

                    // If square is enemy, track it!
                    else if target_square.white != self.white {
                        moves.push(Move {
                            from: self.coord,
                            to: target_square.coord,
                            is_capture: Some(true),
                            en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                        });
                    }
                }
            }
            PieceType::Pawn => {
                // White pawns go up the board, black ones go down.
                let direction: i32 = if self.white {-1} else {1};
                let pawn_starting_coord_y: usize = if self.white {6} else {1};

                
                let y_pos: i32 = self.coord.y as i32 + direction;

                
                /*
                    This condition should be impossible because of how pawn moves work.
                    If a pawn makes it to the top/bottom rank, it promotes and is no longer a pawn.
                    But leaving bounds check just in case.
                 */
                if y_pos < 0 || y_pos >= BOARD_SIZE as i32 {
                    return moves;
                }

                // If pawn reaches first/last rank, we can promote.
                let can_promote: bool;
                if y_pos == 0 || y_pos == BOARD_SIZE as i32 - 1 {
                    can_promote = true;
                } else {
                    can_promote = false;
                }
                
                let mut temporary_moves: Vec<Move> = vec![];

                // Check moves, not captures here.
                let target = &game.board[y_pos as usize][self.coord.x];
                if target.piece_type == PieceType::None {

                    // TODO: Handle pawn promotion.
                    temporary_moves.push(Move {
                        from: self.coord,
                        to: target.coord,
                        is_capture: Some(false),
                        en_pessant_target_coord: None,
                        pawn_promoting_to: None,
                    });

                    // If pawn is on it's starting square, look for double moves.
                    if self.coord.y == pawn_starting_coord_y {
                        let extra_target = &game.board[(y_pos + direction) as usize][self.coord.x];
                        if extra_target.piece_type == PieceType::None {
                            temporary_moves.push(Move {
                                from: self.coord,
                                to: extra_target.coord,
                                is_capture: Some(false),
                                en_pessant_target_coord: Some(Coord {
                                    x: self.coord.x,
                                    y: y_pos as usize,
                                }),
                                pawn_promoting_to: None,
                            });
                        }
                    }
                }

                // Check the left captures.
                if self.coord.x as i32 - 1 >= 0 {
                    let target_coord = Coord {
                        x: self.coord.x - 1,
                        y: y_pos as usize,
                    };

                    if game.en_passant_target == Some(target_coord) {
                        temporary_moves.push(Move {
                            from: self.coord,
                            to: target_coord,
                            is_capture: Some(true),
                            en_pessant_target_coord: None, 
                            pawn_promoting_to: None,
                        });
                    }

                    // See if tile is enemy piece.
                    let piece = game.get_piece_at_coord(&target_coord);
                    if piece.piece_type != PieceType::None && piece.white != self.white {
                        temporary_moves.push(Move {
                            from: self.coord,
                            to: target_coord,
                            is_capture: Some(true),
                            en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                        });
                    }
                }

                // Check the right captures.
                if self.coord.x + 1 < BOARD_SIZE {
                    let target_coord = Coord {
                        x: self.coord.x + 1,
                        y: y_pos as usize,
                    };

                    if game.en_passant_target == Some(target_coord) {
                        temporary_moves.push(Move {
                            from: self.coord,
                            to: target_coord,
                            is_capture: Some(true),
                            en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                        });
                    }

                    // See if tile is enemy piece.
                    let piece = game.get_piece_at_coord(&target_coord);
                    if piece.piece_type != PieceType::None && piece.white != self.white {
                        temporary_moves.push(Move {
                            from: self.coord,
                            to: target_coord,
                            is_capture: Some(true),
                            en_pessant_target_coord: None,
                            pawn_promoting_to: None,
                        });
                    }
                }

                if can_promote {
                    for m in temporary_moves.iter() {
                        for t in PAWN_PROMOTION_CHOICES {
                            let mut copy_move = m.clone();
                            copy_move.pawn_promoting_to = Some(t);
                            moves.push(copy_move);
                        }
                    }
                } else {
                    moves.append(&mut temporary_moves);
                }
            }
            PieceType::None => {
                return moves;
            }
        }

        // Try out every move. See if it's illegal.
        let mut moves_final: Vec<Move> = vec![];
        for m in moves.iter() {
            if !game.does_move_put_self_in_check(m) {
                moves_final.push(m.clone());
            }
        }

        return moves_final;
    }

    pub fn can_castle(&self, game: &Game, dir: CastleSides) -> bool {
        let mut coords_should_be_empty: Vec<Coord> = vec![];

        match dir {
            CastleSides::Short => {
                if self.white && !game.can_white_castle_short {
                    return false;
                }
                if !self.white && !game.can_black_castle_short {
                    return false;
                }
                coords_should_be_empty.push(Coord {
                    x: self.coord.x + 1,
                    y: self.coord.y,
                });
                coords_should_be_empty.push(Coord {
                    x: self.coord.x + 2,
                    y: self.coord.y,
                });
            }
            CastleSides::Long => {
                if self.white && !game.can_white_castle_long {
                    return false;
                }
                if !self.white && !game.can_black_castle_long {
                    return false;
                }
                coords_should_be_empty.push(Coord {
                    x: self.coord.x - 1,
                    y: self.coord.y,
                });
                coords_should_be_empty.push(Coord {
                    x: self.coord.x - 2,
                    y: self.coord.y,
                });
                coords_should_be_empty.push(Coord {
                    x: self.coord.x - 3,
                    y: self.coord.y,
                });
            }
        }

        // 1. Make sure there are empty spaces on the expected tiles.
        for coord in coords_should_be_empty.iter() {
            if game.get_piece_at_coord(coord).piece_type != PieceType::None {
                return false;
            }
        }

        // 2. Make sure no enemy pieces are attacking the intermediary square.
        let intermediary_coord = coords_should_be_empty[0];
        for row in game.board.iter() {
            for piece in row.iter() {
                if
                    piece.piece_type != PieceType::None
                    && piece.white != self.white
                    && piece.is_attacking_coord(&intermediary_coord, game)
                {
                    return false;
                }
            }
        }

        // Otherwise, we can castle!
        return true;
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
                let dx: i32;
                let dy: i32;

                if coord.x > self.coord.x {
                    dx = 1;
                }
                else if coord.x < self.coord.x {
                    dx = -1;
                }
                else {
                    dx = 0;
                }

                if coord.y > self.coord.y {
                    dy = 1;
                }
                else if coord.y < self.coord.y {
                    dy = -1;
                }
                else {
                    dy = 0;
                }

                /* Consider finding a way to not check impossible directions. */
                return self.can_attack_coord_in_direction(game, coord, dx, dy);
            }
            PieceType::Rook => {
                let dx: i32;
                let dy: i32;

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

                // Try to attack the square.
                return self.can_attack_coord_in_direction(game, coord, dx, dy);
            }
            PieceType::Bishop => {
                let dx: i32;
                let dy: i32;

                if coord.x > self.coord.x {
                    dx = 1;
                }
                else if coord.x < self.coord.x {
                    dx = -1;
                }
                else { return false }

                if coord.y > self.coord.y {
                    dy = 1;
                }
                else if coord.y < self.coord.y {
                    dy = -1;
                }
                else { return false }

                /* Consider finding a way to not check impossible directions. */
                return self.can_attack_coord_in_direction(game, coord, dx, dy);
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