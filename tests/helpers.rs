use chess_engine::helpers::str_coord_to_square;

#[test]
fn test_str_coord_to_square() {
    let valid: Vec<&str> = vec!["e2", "f4", "d6", "a8", "h8", "a1", "a8", "h1"];
    let invalid: Vec<&str> = vec!["f", "ssssss", "  ", "", "a0", "h9", "h0", "a9", "y8", "77", "Z1"];

    for s in valid.iter() {
        assert!(str_coord_to_square(s).is_ok(), "Failed to coord {s} correctly.");
    }

    for s in invalid.iter() {
        assert!(str_coord_to_square(s).is_err(), "Parsed coord {s} when it should have failed.");
    }
}