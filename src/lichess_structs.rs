// This will have all the struct definitions we will need to run the bot.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserBasic {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub rating: u32,
    pub provisional: bool,
}

// Implement the Default trait
impl Default for UserBasic {
    fn default() -> Self {
        UserBasic {
            id: String::new(),
            name: String::new(),
            title: None,
            rating: 0,
            provisional: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub r#type: String,
    pub moves: String,
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
    pub status: String,
}

impl GameState {
    pub fn moves_to_vec(&self) -> Vec<String> {
        if self.moves == "" {
            return vec![];
        }

        // Split moves on the space.
        let mut moves_vec: Vec<String> = vec![];
        let mut sp = self.moves.split(" ");

        loop {
            // Read moves until we are all done.
            let m = match sp.next() {
                Some(s) => s.to_string(),
                None => break,
            };

            moves_vec.push(m);
        }

        return moves_vec;
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            r#type: String::new(),
            moves: String::new(),
            wtime: 0,
            btime: 0,
            winc: 0,
            binc: 0,
            status: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameFull {
    pub r#type: String,
    pub id: String,
    pub white: UserBasic,
    pub black: UserBasic,
    pub state: GameState,

    // Lichess API gives us CAMEL CASE. So we fix it.
    #[serde(rename = "initialFen")]
    pub initial_fen: String,
}

// Implement the Default trait for Piece
impl Default for GameFull {
    fn default() -> Self {
        GameFull {
            r#type: String::new(),
            id: String::new(),
            white: UserBasic::default(),
            black: UserBasic::default(),
            state: GameState::default(),
            initial_fen: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    pub r#type: String,
    pub challenge: InnerChallenge,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerChallenge {
    pub id: String,
    pub url: String,
    pub status: String,

    // There are actually a couple other fields here, but we don't care about them.
    pub challenger: UserBasic,

    // Lichess API gives us CAMEL CASE. So we fix it.
    #[serde(rename = "destUser")]
    pub dest_user: UserBasic,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeGameStart {
    pub r#type: String,
    pub game: InnerChallengeGameStart,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerChallengeGameStart {
    pub id: String,
    pub color: String,
    pub fen: String,
    pub source: String,

    #[serde(rename = "hasMoved")]
    pub has_moved: bool,

    #[serde(rename = "isMyTurn")]
    pub is_my_turn: bool,

    #[serde(rename = "lastMove")]
    pub last_move: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatLineEvent {
    pub r#type: String,
    pub username: String,
    pub text: String,
    pub room: String,
}
