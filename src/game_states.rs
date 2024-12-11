#[derive(Clone, PartialEq)]
pub enum GameState {
    InProgress,
    WhiteWins,
    BlackWins,
    Draw,
}

impl GameState {
    pub fn print_game_state(&self) {
        match self {
            Self::InProgress => println!("Game is still in progress."),
            Self::WhiteWins => println!("White wins!"),
            Self::BlackWins => println!("Black wins!"),
            Self::Draw => println!("Game is drawn!"),
        }
    }
}
