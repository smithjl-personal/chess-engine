// Figuring out how bitboards work.
// Define a type that can hold 64 bits of information (unsigned).
// Use u64.

pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn idx(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

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
    pub fn init_magic_numbers(&mut self, mut bishop_magic_numbers: [u64; 64], mut rook_magic_numbers: [u64; 64]) {
        let bishop_relevant_bits: [usize; 64] = [
            6, 5, 5, 5, 5, 5, 5, 6,
            5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 7, 7, 7, 7, 5, 5,
            5, 5, 7, 9, 9, 7, 5, 5,
            5, 5, 7, 9, 9, 7, 5, 5,
            5, 5, 7, 7, 7, 7, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5,
            6, 5, 5, 5, 5, 5, 5, 6,
        ];

        let rook_relevant_bits: [usize; 64] = [
            12, 11, 11, 11, 11, 11, 11, 12,
            11, 10, 10, 10, 10, 10, 10, 11,
            11, 10, 10, 10, 10, 10, 10, 11,
            11, 10, 10, 10, 10, 10, 10, 11,
            11, 10, 10, 10, 10, 10, 10, 11,
            11, 10, 10, 10, 10, 10, 10, 11,
            11, 10, 10, 10, 10, 10, 10, 11,
            12, 11, 11, 11, 11, 11, 11, 12,
        ];

        // For each square on the board.
        for square in 0..64 {
            // Handle rooks.
            let n = self.find_magic_number(square, rook_relevant_bits[square as usize], false);
            println!("0x{:X}ULL", n);
            rook_magic_numbers[square as usize] = n;
        }

        println!("\nXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\n");

        // For each square on the board.
        for square in 0..64 {
            // Handle bishops.
            let n = self.find_magic_number(square, bishop_relevant_bits[square as usize], true);
            println!("0x{:X}ULL", n);
            bishop_magic_numbers[square as usize] = n;
        }
    }
}

pub const NOT_FILE_A: u64 = 18374403900871474942;
pub const NOT_FILE_B: u64 = 18302063728033398269;
pub const NOT_FILE_AB: u64 = 18229723555195321596;
pub const NOT_FILE_G: u64 = 13816973012072644543;
pub const NOT_FILE_H: u64 = 9187201950435737471;
pub const NOT_FILE_GH: u64 = 4557430888798830399;

pub const NOT_RANK_8: u64 = 18446744073709551360;
pub const NOT_RANK_7: u64 = 18446744073709486335;
pub const NOT_RANK_2: u64 = 18374967954648334335;
pub const NOT_RANK_1: u64 = 72057594037927935;

/*
    All Squares: 18446744073709551615

*/

pub struct Constants {
    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],

    pub bishop_magic_numbers: [u64; 64],
    pub rook_magic_numbers: [u64; 64],
}

impl Constants {
    pub fn new() -> Self {
        let mut pawn_attacks: [[u64; 64]; 2] = [[0; 64]; 2];
        let mut knight_attacks: [u64; 64] = [0; 64];
        let mut king_attacks: [u64; 64] = [0; 64];

        let mut bishop_magic_numbers: [u64; 64] = [0; 64];
        let mut rook_magic_numbers: [u64; 64] = [0; 64];


        for square in 0..64 {
            pawn_attacks[Color::White.idx()][square] = mask_pawn_attacks(square as u64, Color::White);
            pawn_attacks[Color::Black.idx()][square] = mask_pawn_attacks(square as u64, Color::Black);

            knight_attacks[square] = mask_knight_attacks(square as u64);
            king_attacks[square] = mask_king_attacks(square as u64);
        }

        let mut magic = MagicNumberHelper::new();
        magic.init_magic_numbers(bishop_magic_numbers, rook_magic_numbers);
        

        return Constants {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop_magic_numbers,
            rook_magic_numbers,
        };
    }
}



pub fn print_bitboard(bitboard: u64) {
    println!("    A   B   C   D   E   F   G   H");
    println!("  |---|---|---|---|---|---|---|---|");
    for rank in 0..8 {
        print!("{} |", 8 - rank);
        for file in 0..8 {
            let square: u64 = rank * 8 + file;
            let calc = get_bit(bitboard, square);
            let populated;
            if calc != 0 {
                populated = 1;
            } else {
                populated = 0;
            }

            print!(" {} |", populated);
        }
        print!(" {}", 8 - rank);
        println!();
        println!("  |---|---|---|---|---|---|---|---|");
    }
    println!("    A   B   C   D   E   F   G   H");
    println!("Bitboard Value: {bitboard}");
}



pub fn mask_pawn_attacks(square: u64, side: Color) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    match side {
        Color::White => {
            // Attacking top right.
            attacks |= (bitboard >> 7) & NOT_FILE_A;

            // Attacking top left.
            attacks |= (bitboard >> 9) & NOT_FILE_H;
        }
        Color::Black => {
            // Attacking bottom right.
            attacks |= (bitboard << 9) & NOT_FILE_A;

            // Attacking bottom left.
            attacks |= (bitboard << 7) & NOT_FILE_H;
        }
    }

    return attacks;
}


pub fn mask_knight_attacks(square: u64) -> u64 {
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


pub fn mask_king_attacks(square: u64) -> u64 {
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


// Should this be an enum?
pub fn str_coord_to_bitboard_pos(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!(
            "Invalid input detected. Expected 2 chars. Got: `{}`.",
            s.len()
        ));
    }

    // We know the length is 2, so we can safely unwrap here.
    let file_str = s.chars().nth(0).unwrap().to_ascii_lowercase();
    let rank_str = s.chars().nth(1).unwrap();

    // Attempt conversion for file letter.
    let file: usize = file_str as usize - 'a' as usize;
    if file >= 8 as usize {
        return Err(format!("Invalid file letter: {}", file_str));
    }

    let rank: usize = match rank_str.to_digit(10) {
        Some(n) => (8 - n).try_into().unwrap(),
        None => return Err(format!("Unable to convert `{}` to a digit.", rank_str)),
    };

    return Ok(rank * 8 + file)
}


// I don't understand how this works yet...
pub fn set_occupancies(index: usize, bits_in_mask: usize, mut attack_mask: u64) -> u64 {
    let mut occupancy: u64 = 0;

    // Loop over bit range in attack mask.
    for count in 0..bits_in_mask {

        // Get LSB of attack mask.
        let square = match get_lsb_index(attack_mask) {
            Ok(v) => v,
            Err(m) => {
                // For now, just panic. This shouldn't happen here?
                panic!("{}", m);
            }
        };

        // Pop the bit.
        attack_mask = pop_bit(attack_mask, square as u64);

        // Make sure occupancy is on the board.
        if index & (1 << count) != 0 {
            occupancy |= 1 << square;
        }
    }

    return occupancy;

}


// Should these be macros? Or something similar?
pub fn get_bit(bitboard: u64, square: u64) -> u64 {
    return bitboard & (1 << square);
}

pub fn set_bit(bitboard: u64, square: u64) -> u64 {
    return bitboard | (1 << square);
}

pub fn pop_bit(bitboard: u64, square: u64) -> u64 {
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

pub fn get_lsb_index(bitboard: u64) -> Result<usize, String> {
    // Below operations will not work on bitboard of `0`.
    if bitboard == 0 {
        return Err("Illegal index requested.".to_string());
    }

    // Get the position of the least-significant bit, using some bit magic!
    let lsb = bitboard & !bitboard + 1;

    // Subtract `1` to populate the trailing bits.
    let populated = lsb - 1;

    return Ok(count_bits(populated));
}