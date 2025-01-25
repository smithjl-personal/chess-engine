#[derive(Clone, PartialEq)]
pub enum GameState {
    InProgress,
    WhiteWins,
    BlackWins,
    Draw,
}

impl GameState {
    pub fn get_game_state_str(&self) -> &str {
        match self {
            Self::InProgress => "Game is still in progress.",
            Self::WhiteWins => "White wins!",
            Self::BlackWins => "Black wins!",
            Self::Draw => "Game is drawn!",
        }
    }
    pub fn print_game_state(&self) {
        println!("{}", self.get_game_state_str());
    }
}
