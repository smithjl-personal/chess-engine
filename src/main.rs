pub mod castle_sides;
pub mod constants;
pub mod coord;
pub mod g; // Something is wrong with rust-analyzer. This is the only way it will pick up the changes right now.
pub mod game_states;
pub mod lichess;
pub mod my_move;
pub mod piece;
pub mod piece_type;
pub mod tests;

use core::str;
// use crate::g::Game;
// use crate::tests::run_all_tests;
// use std::error::Error;
use constants::{INITIAL_GAME_STATE_FEN, LICHESS_CHALLENGER_WHITELIST};
use my_move::Move;
use serde_json;
use std::env;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    run_lichess_bot().await;
}

// TODO: Move these to separate file.
async fn play_lichess_game(game_id: String) {
    let lichess_url = format!("https://lichess.org/api/bot/game/stream/{game_id}");
    let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
        Ok(s) => s,
        Err(e) => {
            println!("Unable to get your bot's API token. Error reading ENV var: `LICHESS_BOT_API_TOKEN`. Make sure it is set!");
            println!("Raw Error: {}", e);
            return;
        }
    };

    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .get(lichess_url)
        .bearer_auth(lichess_auth_token)
        .send()
        .await;

    let mut response = match response_result {
        Ok(r) => r,
        Err(e) => {
            print!("Error: {}", e);
            return;
        }
    };

    //println!("Raw Response: {:#?}", response);

    // This function will run forever, when calling a streamed API.
    let mut lichess_game: lichess::GameFull = lichess::GameFull::default();
    let mut game = g::Game::default();
    let mut is_bot_white: bool = true;
    while let Some(chunk) = response.chunk().await.unwrap() {
        // We just received the '\n' from the API to keep the connection alive. Ignore processing.
        if chunk.len() == 1 {
            println!("Waiting for new data...");
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

        //println!("Looking at below json:\n{}", full_str);

        // Look through the bytes we got, se expect a few possible items here.
        if full_str.contains("\"type\":\"opponentGone\"") {
            println!("Opponent left. Code not set up to handle this.");
            continue;
        } else if full_str.contains("\"type\":\"chatLine\"") {
            let lichess_chat_line_parse_attempt: Result<lichess::ChatLineEvent, serde_json::Error> =
                serde_json::from_str(full_str);
            let chat_event: lichess::ChatLineEvent = match lichess_chat_line_parse_attempt {
                Ok(g) => g,
                Err(e) => {
                    println!("Unable to parse lichess chat line. Error: {}", e);
                    continue;
                }
            };

            println!("We parsed this chat line event: {:#?}", chat_event);

            if chat_event.text == "debug" {
                
                println!("{}", game.get_debug_game_state_str());
                let _ = write_lichess_chat_message(
                    lichess_game.id.to_string(), 
                    "Message recieved. Check the console.".to_string()
                ).await;
            }
            continue;
        } else if full_str.contains("\"type\":\"gameFull\"") {
            println!("Game full event read. Will parse this soon.");
            let lichess_game_parse_attempt: Result<lichess::GameFull, serde_json::Error> =
                serde_json::from_str(full_str);
            lichess_game = match lichess_game_parse_attempt {
                Ok(g) => g,
                Err(e) => {
                    println!("Unable to parse lichess full game. Error: {}", e);
                    continue;
                }
            };
            //println!("Parsed game successfully! {:?}", lichess_game);
            // The first time we load the game, we need go get the game state aligned...
            if lichess_game.initial_fen != "startpos" {
                println!("Game did not start in the initial condition. Cannot continue.");
                break;
            }

            // Should this be hardcoded?
            is_bot_white = lichess_game.white.id == "botmasterj";

            // Import the FEN.
            game.import_fen(INITIAL_GAME_STATE_FEN);

            // Make all the moves.
            // println!(
            //     "Vector of moves, according to licess: {:#?}",
            //     lichess_game.state.moves_to_vec()
            // );
            for str_move in lichess_game.state.moves_to_vec().iter() {
                let external_move: Move = Move::str_to_move(&str_move).unwrap(); // Risky!

                if !game.legal_moves.contains(&external_move) {
                    println!("Game thinks move {} is illegal...", external_move);
                }

                // Grab the move from the game, it has more data (capture, en-passant, etc).
                let mut user_move: Option<Move> = None;
                for m in game.legal_moves.iter() {
                    if m == &external_move {
                        user_move = Some(m.clone());
                        break;
                    }
                }

                match user_move {
                    Some(m) => game.make_move_and_update_state(&m),
                    None => {
                        println!("Tried to make 'illegal' move...");
                        //self.print_all_legal_moves();
                        continue;
                    }
                }

                //let internal_move

                //game.make_move_and_update_state(&m, Some(true));
                // println!("{}", game.export_fen());
            }
            // println!("Stopping due to testing.");
            // break;

            // Let the rest below handle the rest. Game state is now set.
        } else if full_str.contains("\"type\":\"gameState\"") {
            let lichess_game_state_parse_attempt: Result<lichess::GameState, serde_json::Error> =
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
            let last_move = Move::str_to_move(&last_move_str);
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
            continue;
        }

        // We know it is our turn. Run minimax to find a good move.
        let bot_move = game.get_bot_move();
        println!("Bot thinks we should play: {}", bot_move);

        // Try to make the move.
        // bad code, but going fast...
        let move_result =
            make_lichess_move(lichess_game.id.to_string(), bot_move.to_string()).await;
        if move_result.is_err() {
            println!(
                "Something went wrong when trying to make the move...\n{:?}",
                move_result.err()
            );
            break;
        }
    }
}

async fn make_lichess_move(game_id: String, r#move: String) -> Result<(), String> {
    let lichess_url = format!("https://lichess.org/api/bot/game/{game_id}/move/{move}");
    let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
        Ok(s) => s,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .post(lichess_url)
        .bearer_auth(lichess_auth_token)
        .send()
        .await;

    let mut response = match response_result {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    return Ok(());
}

async fn run_lichess_bot() {
    let lichess_event_url = "https://lichess.org/api/stream/event";
    let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
        Ok(s) => s,
        Err(e) => {
            println!("Unable to get your bot's API token. Error reading ENV var: `LICHESS_BOT_API_TOKEN`. Make sure it is set!");
            println!("Raw Error: {}", e);
            return;
        }
    };

    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .get(lichess_event_url)
        .bearer_auth(lichess_auth_token)
        .send()
        .await;

    let mut response = match response_result {
        Ok(r) => r,
        Err(e) => {
            print!("Error: {}", e);
            return;
        }
    };

    // The API will stream us data.
    while let Some(chunk) = response.chunk().await.unwrap() {
        // We just received the '\n' from the API to keep the connection alive. Ignore processing.
        if chunk.len() == 1 {
            println!("Activity: None to parse.");
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

        println!("Looking at below json:\n{}", full_str);

        // Look through the bytes we got, se expect a few possible items here.
        if full_str.contains("\"type\":\"challenge\"") {
            // Attempt to parse accordingly.
            let lichess_challenge_raw: Result<lichess::Challenge, serde_json::Error> = serde_json::from_str(full_str);
            let lichess_challenge = match lichess_challenge_raw {
                Ok(c) => c.challenge,
                Err(e) => {
                    println!("Unable to parse lichess challenge. Error: {}", e);
                    continue;
                }
            };
            println!("We parsed the challenge...\n{:#?}", lichess_challenge);

            // For now, only accept challenges from me.
            if !LICHESS_CHALLENGER_WHITELIST.contains(&lichess_challenge.challenger.name.as_str()) {
                println!("Challenger {} is not on the whitelist. Ignoring for now.", lichess_challenge.challenger.name);
                continue;
            }

            let _ = accept_lichess_challenge(lichess_challenge.id).await;
        } else if full_str.contains("\"type\":\"gameStart\"") {
            println!("Game start event. Not coded to handle this yet.");
            // Attempt to parse accordingly.
            let lichess_challenge_start: Result<lichess::ChallengeGameStart, serde_json::Error> = serde_json::from_str(full_str);
            let lichess_game_full = match lichess_challenge_start {
                Ok(c) => c.game,
                Err(e) => {
                    println!("Unable to parse lichess challenge. Error: {}", e);
                    continue;
                }
            };

            // TODO: Maybe run the game in another thread???
            println!("We parsed this incoming game, handing off to game handler...\n{:#?}", lichess_game_full);
            let _ = play_lichess_game(lichess_game_full.id).await;
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

async fn accept_lichess_challenge(game_id: String) -> Result<(), String> {
    let lichess_url = format!("https://lichess.org/api/challenge/{game_id}/accept");
    let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
        Ok(s) => s,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .post(lichess_url)
        .bearer_auth(lichess_auth_token)
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

async fn write_lichess_chat_message(game_id: String, message: String) -> Result<(), String> {
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