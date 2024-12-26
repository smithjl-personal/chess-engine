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
pub mod bitboard;

#[tokio::main]
async fn main() {
    //let _ = lichess::main().await;
    //let _ = tests::test_performance_of_minimax();

    //let mut bb: u64 = 18446744073709551615; // All squares.
    //let mut bb: u64 = 0; // No squares.
    // bitboard::print_bitboard(
    //     18446744073709551615 & bitboard::NOT_RANK_8 & bitboard::NOT_RANK_7
    // );

    // bitboard::print_bitboard(
    //     bitboard::NOT_FILE_AB
    // );

    let mut c = bitboard::Constants::new();

    // Get a coord to test.
    // let mut bb: u64 = 0; // No squares.
    // let test_coord = bitboard::str_coord_to_bitboard_pos("d4").unwrap();
    // bb = bitboard::set_bit(bb, test_coord as u64);
    // println!("Piece located at:");
    // bitboard::print_bitboard(bb);

    let mut blocker_bb: u64 = 0;
    // blocker_bb = bitboard::set_bit(
    //     blocker_bb,
    //     bitboard::str_coord_to_bitboard_pos("d7").unwrap() as u64
    // );
    // blocker_bb = bitboard::set_bit(
    //     blocker_bb,
    //     bitboard::str_coord_to_bitboard_pos("d2").unwrap() as u64
    // );
    // blocker_bb = bitboard::set_bit(
    //     blocker_bb,
    //     bitboard::str_coord_to_bitboard_pos("b4").unwrap() as u64
    // );
    // blocker_bb = bitboard::set_bit(
    //     blocker_bb,
    //     bitboard::str_coord_to_bitboard_pos("g4").unwrap() as u64
    // );

    // let bit_count = bitboard::count_bits(blocker_bb);
    // println!("Blocker bitboard (has {bit_count} blockers)");
    // bitboard::print_bitboard(blocker_bb);

    // other tests...
    //let test_bb = blocker_bb & !blocker_bb + 1;
    // println!("Index of LSB: {}", bitboard::get_lsb_index(blocker_bb).unwrap());
    //bitboard::print_bitboard(get_lsb_index(blocker_bb));



    // let piece_attacks = bitboard::dynamic_rook_attacks(test_coord as u64, blocker_bb);
    // println!("Piece attacking:");
    // bitboard::print_bitboard(piece_attacks);

    // Bitboard for all squares: 18446744073709551615
    //bitboard::print_bitboard(18446744073709551615);
    // for tile in 0..64 {
    //     bitboard::print_bitboard(
    //         bitboard::mask_rook_attacks(tile as u64)
    //     );
    // }

    // Make an attack mask.
    // let mask = bitboard::mask_rook_attacks(
    //     str_coord_to_bitboard_pos("a1").unwrap() as u64
    // );
    // let occupancy = bitboard::set_occupancies(2, count_bits(mask), mask);
    // print_bitboard(occupancy);


    // bitboard::print_bitboard(
    //     get_magic_number(&mut c)
    // );
    //bitboard::init_magic_numbers(&mut c);
}
