use chess_engine::r#move::Move;


#[test]
fn test_str_to_move() {
    let mut move_str: &str;
    let mut parsed_move: Result<Move, String>;
    let mut unwrapped_move: Move;

    // Basic string move parsing.
    move_str = "e2e4";
    parsed_move = Move::str_to_move(move_str);
    assert!(parsed_move.is_ok(), "Failed to parse move {move_str} correctly.");
    unwrapped_move = parsed_move.unwrap();
    assert!(unwrapped_move.from_square == 52);
    assert!(unwrapped_move.to_square == 36);
    assert!(unwrapped_move.pawn_promoting_to.is_none());

    // Promotion.
    move_str = "e7e8q";
    parsed_move = Move::str_to_move(move_str);
    assert!(parsed_move.is_ok(), "Failed to parse move {move_str} correctly.");
    unwrapped_move = parsed_move.unwrap();
    assert!(unwrapped_move.from_square == 12);
    assert!(unwrapped_move.to_square == 4);
    assert!(unwrapped_move.pawn_promoting_to.is_some());

    // Should fail. Fix TODO.
    // move_str = "a0a0";
    // parsed_move = Move::str_to_move(move_str);
    // print!("{parsed_move:#?}");
    // assert!(parsed_move.is_err(), "Parsed invalid move {move_str} when it should have failed.");
}