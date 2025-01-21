use crate::constants;
use crate::bitboard;

// Consider moving these to true constants file? But they are only used here...
pub const NOT_FILE_A: u64 = 18374403900871474942;
pub const NOT_FILE_B: u64 = 18302063728033398269;
pub const NOT_FILE_AB: u64 = 18229723555195321596;
pub const NOT_FILE_G: u64 = 13816973012072644543;
pub const NOT_FILE_H: u64 = 9187201950435737471;
pub const NOT_FILE_GH: u64 = 4557430888798830399;

pub struct Constants {
    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],
    pub bishop_attacks: Vec<Vec<u64>>, // [64][512]
    pub rook_attacks: Vec<Vec<u64>>,   // [64][4096]
}

impl Constants {
    pub fn new() -> Self {
        let mut pawn_attacks: [[u64; 64]; 2] = [[0; 64]; 2];
        let mut knight_attacks: [u64; 64] = [0; 64];
        let mut king_attacks: [u64; 64] = [0; 64];

        // These are too big to put on the stack.
        let mut bishop_attacks: Vec<Vec<u64>> = vec![vec![0; 512]; 64];
        let mut rook_attacks: Vec<Vec<u64>> = vec![vec![0; 4096]; 64];

        for square in 0..64 {
            pawn_attacks[bitboard::Color::White.idx()][square] =
                mask_pawn_attacks(square, bitboard::Color::White);
            pawn_attacks[bitboard::Color::Black.idx()][square] =
                mask_pawn_attacks(square, bitboard::Color::Black);

            knight_attacks[square] = mask_knight_attacks(square);
            king_attacks[square] = mask_king_attacks(square);
        }

        init_slider_attacks(true, &mut bishop_attacks, &mut rook_attacks);
        init_slider_attacks(false, &mut bishop_attacks, &mut rook_attacks);

        return Constants {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop_attacks,
            rook_attacks,
        };
    }
}


pub fn init_slider_attacks(
    is_bishop: bool,
    bishop_attacks: &mut Vec<Vec<u64>>,
    rook_attacks: &mut Vec<Vec<u64>>,
) {
    for square in 0..64 {
        let attack_mask;
        if is_bishop {
            attack_mask = constants::BISHOP_MASKED_ATTACKS[square];
        } else {
            attack_mask = constants::ROOK_MASKED_ATTACKS[square];
        }

        let relevant_bits_count = count_bits(attack_mask);
        let occupancy_indicies: usize = 1 << relevant_bits_count;
        for index in 0..occupancy_indicies {
            if is_bishop {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::BISHOP_RELEVANT_BITS[square];
                bishop_attacks[square][magic_index as usize] =
                    dynamic_bishop_attacks(square as u64, occupancy);
            } else {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::ROOK_RELEVANT_BITS[square];
                rook_attacks[square][magic_index as usize] =
                    dynamic_rook_attacks(square as u64, occupancy);
            }
        }
    }
}


// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_bishop_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 6 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 6 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file -= 1;
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while rank >= 1 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank >= 1 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file -= 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_bishop_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 7 && file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        // Break the loop when we would subtract underflow.
        rank += 1;
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_rook_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 6 {
        attacks |= 1 << (rank * 8 + file);
        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while file >= 1 {
        attacks |= 1 << (rank * 8 + file);
        file -= 1;
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    while rank >= 1 {
        attacks |= 1 << (rank * 8 + file);
        rank -= 1;
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 6 {
        attacks |= 1 << (rank * 8 + file);
        rank += 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_rook_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
    }

    return attacks;
}


// I don't understand how this works yet...
pub fn set_occupancies(index: usize, bits_in_mask: usize, mut attack_mask: u64) -> u64 {
    let mut occupancy: u64 = 0;

    // Loop over bit range in attack mask.
    for count in 0..bits_in_mask {
        // Get LSB of attack mask.
        let square = match get_lsb_index(attack_mask) {
            Some(v) => v,
            None => {
                panic!("Unable to set occupancies, unexpected value for `get_lsb_index`.");
            }
        };

        // Pop the bit.
        attack_mask = pop_bit(attack_mask, square);

        // Make sure occupancy is on the board.
        if index & (1 << count) != 0 {
            occupancy |= 1 << square;
        }
    }

    return occupancy;
}



pub fn mask_pawn_attacks(square: usize, side: bitboard::Color) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    match side {
        bitboard::Color::White => {
            // Attacking top right.
            attacks |= (bitboard >> 7) & NOT_FILE_A;

            // Attacking top left.
            attacks |= (bitboard >> 9) & NOT_FILE_H;
        }
        bitboard::Color::Black => {
            // Attacking bottom right.
            attacks |= (bitboard << 9) & NOT_FILE_A;

            // Attacking bottom left.
            attacks |= (bitboard << 7) & NOT_FILE_H;
        }
    }

    return attacks;
}

pub fn mask_knight_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 10) & NOT_FILE_GH; // Up 1 Left 2
    attacks |= (bitboard >> 17) & NOT_FILE_H; // Up 2 Left 1

    attacks |= (bitboard >> 6) & NOT_FILE_AB; // Up 1 Right 2
    attacks |= (bitboard >> 15) & NOT_FILE_A; // Up 2 Right 1

    attacks |= (bitboard << 6) & NOT_FILE_GH; // Down 1 Left 2
    attacks |= (bitboard << 15) & NOT_FILE_H; // Down 2 Left 1

    attacks |= (bitboard << 10) & NOT_FILE_AB; // Down 1 Right 2
    attacks |= (bitboard << 17) & NOT_FILE_A; // Down 2 Right 1

    return attacks;
}

pub fn mask_king_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 9) & NOT_FILE_H; // Up 1 Left 1
    attacks |= bitboard >> 8; // Up 1
    attacks |= (bitboard >> 7) & NOT_FILE_A; // Up 1 Right 1

    attacks |= (bitboard >> 1) & NOT_FILE_H; // Left 1
    attacks |= (bitboard << 1) & NOT_FILE_A; // Right 1

    attacks |= (bitboard << 7) & NOT_FILE_H; // Down 1 Left 1
    attacks |= bitboard << 8; // Down 1
    attacks |= (bitboard << 9) & NOT_FILE_A; // Down 1 Right 1

    return attacks;
}





// Should these be macros? Or something similar?
pub fn get_bit(bitboard: u64, square: usize) -> u64 {
    return bitboard & (1 << square);
}

pub fn set_bit(bitboard: u64, square: usize) -> u64 {
    return bitboard | (1 << square);
}

pub fn pop_bit(bitboard: u64, square: usize) -> u64 {
    if get_bit(bitboard, square) != 0 {
        return bitboard ^ (1 << square);
    } else {
        return bitboard;
    }
}

pub fn count_bits(mut bitboard: u64) -> usize {
    let mut bit_count = 0;

    while bitboard != 0 {
        bit_count += 1;

        // Reset the least significant bit, once per iteration until there are no active bits left.
        bitboard &= bitboard - 1;
    }

    return bit_count;
}

pub fn get_lsb_index(bitboard: u64) -> Option<usize> {
    // Below operations will not work on bitboard of `0`.
    if bitboard == 0 {
        return None;
    }

    // Get the position of the least-significant bit, using some bit magic!
    let lsb = bitboard & !bitboard + 1;

    // Subtract `1` to populate the trailing bits.
    let populated = lsb - 1;

    return Some(count_bits(populated));
}



/*
    // This is the code we used to generate magic numbers. We don't need to run it again, but it should remain. Somewhere.
    struct MagicNumberHelper {
        pub state: u32,
    }

    impl MagicNumberHelper {
        pub fn new() -> Self {
            return MagicNumberHelper {
                state: 1804289383, // Seed for our Psuedo-RNG.
            }
        }

        fn get_random_number_u32(&mut self) -> u32 {
            // Get current state. This is our seed.
            let mut n = self.state;

            // XOR Shift Algorithm to get a random number.
            n ^= n << 13;
            n ^= n >> 17;
            n ^= n << 5;

            // Update the state.
            self.state = n;

            return n;
        }

        fn get_random_number_u64(&mut self) -> u64 {
            // Define some random numbers. We want the 16 bits from MSB1 side.
            let n1: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n2: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n3: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n4: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;

            // Return them with fanciness.
            return n1 | (n2 << 16) | (n3 << 32) | (n4 << 48);
        }

        fn get_magic_number(&mut self) -> u64 {
            return self.get_random_number_u64() & self.get_random_number_u64() & self.get_random_number_u64();
        }

        // Magic numbers?
        fn find_magic_number(&mut self, square: u64, relevant_bits: usize, is_bishop: bool) -> u64 {

            // Init occupancies, attack tables, and used attacks.
            let mut occupancies: [u64; 4096] = [0; 4096];
            let mut attacks: [u64; 4096] = [0; 4096];
            let mut used_attacks: [u64; 4096];

            // Init attack mask, either bishop or rook.
            let attack_mask: u64;
            if is_bishop {
                attack_mask = mask_bishop_attacks(square);
            } else {
                attack_mask = mask_rook_attacks(square);
            }

            // Init occupancy indicies.
            let occupancy_indicies: usize = 1 << relevant_bits;

            // Loop over occupancy indicies.
            for index in 0..occupancy_indicies {
                occupancies[index] = set_occupancies(index, relevant_bits, attack_mask);

                if is_bishop {
                    attacks[index] = dynamic_bishop_attacks(square, occupancies[index]);
                } else {
                    attacks[index] = dynamic_rook_attacks(square, occupancies[index]);
                }
            }

            // Now test for magic numbers. Should not take too long to run though!
            for _ in 0..1_000_000_000 {
                // Generate magic number candidate.
                let magic_number = self.get_magic_number();

                // This should be safe from overflow?
                let (temp, _) = attack_mask.overflowing_mul(magic_number);
                let to_check = temp & 0xFF00000000000000;

                // Go next if we don't have enough bits.
                if count_bits(to_check) < 6 {
                    continue;
                }

                // Clear out any used attacks from previous iteration.
                used_attacks = [0; 4096];
                let mut has_failed: bool = false;
                for index in 0..occupancy_indicies {

                    // Overflow safe?
                    let (temp, _) = occupancies[index].overflowing_mul(magic_number);
                    let magic_index = (temp >> (64 - relevant_bits)) as usize;

                    if used_attacks[magic_index] == 0 {
                        used_attacks[magic_index] = attacks[index];
                    } else if used_attacks[magic_index] != attacks[index] {
                        has_failed = true;
                        break;
                    }
                }

                if !has_failed {
                    return magic_number;
                }
            }

            panic!("Unable to find magic number, oh no!");
        }

        // This function will print all the magic numbers, then they can be copied for later use in other parts of the program.
        pub fn init_magic_numbers(&mut self, bishop_magic_numbers: &mut [u64; 64], rook_magic_numbers: &mut [u64; 64]) {
            // For each square on the board.
            for square in 0..64 {
                // Handle rooks.
                let n = self.find_magic_number(square, constants::ROOK_RELEVANT_BITS[square as usize], false);
                // println!("0x{:X}ULL", n);
                rook_magic_numbers[square as usize] = n;
            }

            println!("\nXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\n");

            // For each square on the board.
            for square in 0..64 {
                // Handle bishops.
                let n = self.find_magic_number(square, constants::BISHOP_RELEVANT_BITS[square as usize], true);
                // println!("0x{:X}ULL", n);
                bishop_magic_numbers[square as usize] = n;
            }
        }
    }
*/
