use std::env;
use core::str;
use crate::{constants, g, lichess_structs, my_move};
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::Mutex;


pub async fn main() {
    //let mut api = LichessAPI::default();
    let api = Arc::new(Mutex::new(LichessAPI::default()));

    // Handle init errors.
    // let init_result = api.init();
    let init_result = api.lock().await.init();
    match init_result {
        Ok(_) => (),
        Err(e) => {
            println!("There was an error initializing your bot.\n{e}");
            return;
        }
    }

    // Run the bot.
    // api.run().await;
    api.lock().await.run().await;
}


pub struct LichessAPI {
    bearer_auth_token: String,
    // TODO: Track how many games are running at once? Maybe in a vector?
}

impl Default for LichessAPI {
    fn default() -> Self {
        LichessAPI {
            bearer_auth_token: String::new(),
        }
    }
}

impl LichessAPI {
    fn init(&mut self) -> Result<(), String> {
        // Try to get the bearer auth token.
        let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("Error reading ENV var: `LICHESS_BOT_API_TOKEN`. Make sure it is set! Detail: {e}"));
            }
        };
        self.bearer_auth_token = lichess_auth_token;

        return Ok(());
    }

    async fn play_game(&self, game_id: String) {
        let lichess_url = format!("https://lichess.org/api/bot/game/stream/{game_id}");
        let client: reqwest::Client = reqwest::Client::new();
        let response_result: Result<reqwest::Response, reqwest::Error> = client
            .get(lichess_url)
            .bearer_auth(&self.bearer_auth_token)
            .send()
            .await;

        // TODO: Handle errors with game stream?
        let mut response = match response_result {
            Ok(r) => r,
            Err(e) => {
                print!("Error: {}", e);
                return;
            }
        };

        // This function will run forever, when calling a streamed API.
        let mut lichess_game: lichess_structs::GameFull = lichess_structs::GameFull::default();
        let mut game = g::Game::default();
        let mut is_bot_white: bool = true;
        while let Some(chunk) = response.chunk().await.unwrap() {
            // We just received the '\n' from the API to keep the connection alive. Ignore processing.
            if chunk.len() == 1 {
                println!("Play Game Thread: Waiting for new data...");
                continue;
            }

            // Attempt to convert bytes to string.
            let full_str = match str::from_utf8(&chunk) {
                Ok(s) => s,
                Err(e) => {
                    println!("Unable to convert byte array to string (utf8). Error: {e}");
                    continue;
                }
            };

            // Look through the bytes we got, se expect a few possible items here.
            if full_str.contains("\"type\":\"opponentGone\"") {
                println!("Opponent left. Code not set up to handle this.");
                continue;
            } else if full_str.contains("\"type\":\"chatLine\"") {
                let lichess_chat_line_parse_attempt: Result<lichess_structs::ChatLineEvent, serde_json::Error> =
                    serde_json::from_str(full_str);
                let chat_event: lichess_structs::ChatLineEvent = match lichess_chat_line_parse_attempt {
                    Ok(g) => g,
                    Err(e) => {
                        println!("Unable to parse lichess chat line. Error: {e}");
                        continue;
                    }
                };

                // If player types debug in the chat, print some info to the screen.
                if chat_event.text == "debug" {
                    println!("{}", game.get_debug_game_state_str());
                    let _ = self.write_chat_message(
                        lichess_game.id.to_string(), 
                        "Message recieved. Check the console.".to_string()
                    ).await;
                }
                continue;
            } else if full_str.contains("\"type\":\"gameFull\"") {
                println!("Game full event read. Will parse this soon.");
                let lichess_game_parse_attempt: Result<lichess_structs::GameFull, serde_json::Error> =
                    serde_json::from_str(full_str);
                lichess_game = match lichess_game_parse_attempt {
                    Ok(g) => g,
                    Err(e) => {
                        println!("Unable to parse lichess full game. Error: {}", e);
                        continue;
                    }
                };

                // The first time we load the game, we need go get the game state aligned...
                if lichess_game.initial_fen != "startpos" {
                    println!("Game did not start in the initial condition. Cannot continue.");
                    break;
                }

                is_bot_white = lichess_game.white.id == constants::LICHESS_BOT_USERNAME;

                // Import the FEN.
                game.import_fen(constants::INITIAL_GAME_STATE_FEN);

                // Make all the moves.
                for str_move in lichess_game.state.moves_to_vec().iter() {
                    let external_move: my_move::Move = my_move::Move::str_to_move(&str_move).unwrap(); // Risky!

                    if !game.legal_moves.contains(&external_move) {
                        println!("Game thinks move {} is illegal... cannot proceed.", external_move);
                        break;
                    }

                    // Grab the move from the game, it has more data (capture, en-passant, etc).
                    let mut user_move: Option<&my_move::Move> = None;
                    for m in game.legal_moves.iter() {
                        if m == &external_move {
                            user_move = Some(m);
                            break;
                        }
                    }

                    match user_move {
                        // TODO: Fix this unneeded clone. Something is not done correctly here. Figure it out.
                        Some(m) => game.make_move_and_update_state(&m.clone()),
                        None => {
                            println!("Tried to make 'illegal' move...");
                            continue;
                        }
                    }
                }
                // Let the rest below handle the rest. Game state is now set.
            } else if full_str.contains("\"type\":\"gameState\"") {
                let lichess_game_state_parse_attempt: Result<lichess_structs::GameState, serde_json::Error> =
                    serde_json::from_str(full_str);
                let lichess_game_state = match lichess_game_state_parse_attempt {
                    Ok(g) => g,
                    Err(e) => {
                        println!("Unable to parse lichess game state. Error: {}", e);
                        continue;
                    }
                };

                // Someone made a move. Update the local copy of our board.
                let moves = lichess_game_state.moves_to_vec();
                let last_move_str = match moves.last() {
                    Some(m) => m as &str,
                    None => {
                        println!("Cannot get the most recent move, none exist!");
                        continue;
                    }
                };

                lichess_game.state = lichess_game_state;

                // TODO: Check for resignation?

                // Handle errors later...
                let last_move = my_move::Move::str_to_move(&last_move_str);
                game.make_move_and_update_state(&last_move.unwrap());

                // Print our evaluation after each move.
                println!("Our evaluation of the position: {}", game.evaluate_board());
            } else {
                println!("Unexpected event type. See what went wrong.\n{}", full_str);
                continue;
            }

            // If we reach this point, see if it's our turn.
            if game.white_to_move != is_bot_white {
                println!("It is the opponents turn. Waiting.");
                continue;
            }

            if lichess_game.state.status != "started" {
                println!("Game is over...? {}", lichess_game.state.status);
                break;
            }

            // We know it is our turn. Run minimax to find a good move.
            let bot_move = game.get_bot_move();
            println!("Bot thinks we should play: {}", bot_move);

            // Try to make the move.
            // TODO: Properly handle errors here.
            let move_result =
                self.make_move(lichess_game.id.to_string(), bot_move.to_string()).await;
            if move_result.is_err() {
                println!(
                    "Something went wrong when trying to make the move...\n{:?}",
                    move_result.err()
                );
                break;
            }
        }
    }

    async fn make_move(&self, game_id: String, r#move: String) -> Result<(), String> {
        let lichess_url = format!("https://lichess.org/api/bot/game/{game_id}/move/{move}");
        let client: reqwest::Client = reqwest::Client::new();
        let response_result: Result<reqwest::Response, reqwest::Error> = client
            .post(lichess_url)
            .bearer_auth(&self.bearer_auth_token)
            .send()
            .await;

        // TODO: Properly handle errors from the API.
        let _ = match response_result {
            Ok(r) => r,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        return Ok(());
    }

    async fn run(&self) {
        let lichess_event_url = "https://lichess.org/api/stream/event";
        let client: reqwest::Client = reqwest::Client::new();
        let response_result: Result<reqwest::Response, reqwest::Error> = client
            .get(lichess_event_url)
            .bearer_auth(&self.bearer_auth_token)
            .send()
            .await;

        let mut response = match response_result {
            Ok(r) => r,
            Err(e) => {
                print!("Error: {}", e);
                return;
            }
        };

        // Wrap `self` in Arc<Mutex<T>> for safe shared ownership
        //let thread_safe_clone = Arc::new(Mutex::new(self.clone()));

        // The API will stream us data.
        while let Some(chunk) = response.chunk().await.unwrap() {
            // We just received the '\n' from the API to keep the connection alive. Ignore processing.
            if chunk.len() == 1 {
                println!("Main Thread Activity: None to parse.");
                continue;
            }

            // Attempt to convert bytes to string.
            let full_str = match str::from_utf8(&chunk) {
                Ok(s) => s,
                Err(e) => {
                    println!(
                        "Unable to convert byte array to string (utf8). Error: {}",
                        e
                    );
                    continue;
                }
            };

            // Look through the bytes we got, se expect a few possible items here.
            if full_str.contains("\"type\":\"challenge\"") {
                // Attempt to parse accordingly.
                let lichess_challenge_raw: Result<lichess_structs::Challenge, serde_json::Error> = serde_json::from_str(full_str);
                let lichess_challenge = match lichess_challenge_raw {
                    Ok(c) => c.challenge,
                    Err(e) => {
                        println!("Unable to parse lichess challenge. Error: {}", e);
                        continue;
                    }
                };

                // For now, only accept challenges from me.
                if !constants::LICHESS_CHALLENGER_WHITELIST.contains(&lichess_challenge.challenger.name.as_str()) {
                    println!("Challenger {} is not on the whitelist. Ignoring for now.", lichess_challenge.challenger.name);
                    continue;
                }

                // Accept the challenge! This will send another event to this function on success.
                let _ = self.accept_challenge(lichess_challenge.id).await;
            } else if full_str.contains("\"type\":\"gameStart\"") {
                // Attempt to parse accordingly.
                let lichess_challenge_start: Result<lichess_structs::ChallengeGameStart, serde_json::Error> = serde_json::from_str(full_str);
                let lichess_game_full = match lichess_challenge_start {
                    Ok(c) => c.game,
                    Err(e) => {
                        println!("Unable to parse lichess challenge. Error: {}", e);
                        continue;
                    }
                };

                // TODO: Fix threading here...
                // tokio::spawn(async {
                //     // Process each socket concurrently.
                //     //process(socket).await
                //     println!("Spawning thread...");
                //     self.play_game(lichess_game_full.id).await;
                // });

                println!("We parsed this incoming game, handing off to game handler...\n{:#?}", lichess_game_full);
                self.play_game(lichess_game_full.id).await;
                continue;
            } else if full_str.contains("\"type\":\"gameFinish\"") {
                println!("Game finish event. Not coded to handle this yet.");
                continue;
            } else if full_str.contains("\"type\":\"challengeCanceled\"") {
                println!("Challenge cancelled event. Not coded to handle this yet.");
                continue;
            } else if full_str.contains("\"type\":\"challengeDeclined\"") {
                println!("Challenge declined event. Not coded to handle this yet.");
                continue;
            } else {
                println!("Unexpected event type. See what went wrong.\n{}", full_str);
                continue;
            }
        }
    }

    async fn accept_challenge(&self, game_id: String) -> Result<(), String> {
        let lichess_url = format!("https://lichess.org/api/challenge/{game_id}/accept");
        let client: reqwest::Client = reqwest::Client::new();
        let response_result: Result<reqwest::Response, reqwest::Error> = client
            .post(lichess_url)
            .bearer_auth(&self.bearer_auth_token)
            .send()
            .await;

        let _ = match response_result {
            Ok(r) => r,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        return Ok(());
    }

    async fn write_chat_message(&self, game_id: String, message: String) -> Result<(), String> {
        let lichess_url = format!("https://lichess.org/api/bot/game/{game_id}/chat");
        let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
            Ok(s) => s,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        let mut params = HashMap::new();
        params.insert("room", "player"); // player/spectator
        params.insert("text", &message);

        let client: reqwest::Client = reqwest::Client::new();
        let response_result: Result<reqwest::Response, reqwest::Error> = client
            .post(lichess_url)
            .bearer_auth(lichess_auth_token)
            .form(&params)
            .send()
            .await;

        println!("api respond to making message... {:#?}", response_result);
        let parsed = match response_result {
            Ok(r) => r,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        println!("text: {:#?}", parsed.text().await);

        return Ok(());
    }

}