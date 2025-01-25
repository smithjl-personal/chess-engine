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


// Debugging minimax.
pub fn debug_depth_to_tabs(mut depth: u32) -> String {
    let mut result = String::new();

    while depth > 0 {
        result.push('\t');
        depth -= 1;
    }

    return result;
}

pub fn square_to_coord(square: usize) -> String {
    let rank = 8 - (square / 8);
    let file_number = square % 8;
    let file_char = ('a' as u8 + file_number as u8) as char;

    return format!("{}{}", file_char, rank);
}

pub fn print_bitboard(bitboard: u64) {
    println!("    A   B   C   D   E   F   G   H");
    println!("  |---|---|---|---|---|---|---|---|");
    for rank in 0..8 {
        print!("{} |", 8 - rank);
        for file in 0..8 {
            let square: usize = rank * 8 + file;
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


// Should this be an enum?
pub fn str_coord_to_square(s: &str) -> Result<usize, String> {
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

    return Ok(rank * 8 + file);
}

