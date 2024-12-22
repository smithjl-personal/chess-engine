use std::env;
use core::str;
use crate::{constants, g, lichess_structs, my_move};
use std::collections::HashMap;

pub async fn main() -> Result<(), String> {
    // Try to get the bearer auth token.
    let lichess_auth_token = match env::var("LICHESS_BOT_API_TOKEN") {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Error reading ENV var: `LICHESS_BOT_API_TOKEN`. Make sure it is set! Detail: {e}"));
        }
    };

    println!("Calling run function for Lichess Bot...");
    run(&lichess_auth_token).await;

    return Ok(());
}

async fn play_game(token: &str, game_id: &str, fen: &str) {
    let lichess_url = format!("https://lichess.org/api/bot/game/stream/{game_id}");
    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .get(lichess_url)
        .bearer_auth(token)
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
                let _ = write_chat_message(
                    token,
                    &lichess_game.id, 
                    "Message recieved. Check the console."
                ).await;
            }
            continue;
        } else if full_str.contains("\"type\":\"gameFull\"") {
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

            // Import the FEN, let the rest below handle the rest. Game state is now set.
            game.import_fen(fen);
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
            println!("It is the opponents turn. Waiting for our turn.");
            continue;
        }

        if lichess_game.state.status != "started" {
            println!("Game is over by: {}", lichess_game.state.status);
            break;
        }

        // We know it is our turn. Run minimax to find a good move.
        let bot_move = game.get_bot_move();
        println!("Bot thinks we should play: {}", bot_move);

        // Try to make the move.
        let move_result = make_move(
            token,
            &lichess_game.id,
            &bot_move.to_string()).await;

        // Handle errors in the console.
        let _ = match move_result {
            Ok(_) => (),
            Err(e) => {
                println!("{e}");
                break;
            }
        };
    }
}

async fn make_move(token: &str, game_id: &str, r#move: &str) -> Result<(), String> {
    let lichess_url = format!("https://lichess.org/api/bot/game/{game_id}/move/{move}");
    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .post(lichess_url)
        .bearer_auth(token)
        .send()
        .await;

    let parsed_response = match response_result {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    if parsed_response.status() != 200 {
        return Err(format!("Something went wrong trying to make a move: {}.\nAPI response: {:#?}.", r#move, parsed_response.text().await));
    }

    return Ok(());
}

async fn run(token: &str) {
    let lichess_event_url = "https://lichess.org/api/stream/event";
    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .get(lichess_event_url)
        .bearer_auth(token)
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
            println!("Main Thread: Waiting for new data...");
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
            let _ = accept_challenge(token, &lichess_challenge.id).await;
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

            // Make a copy of the token to pass to the thread.
            let cloned_token = token.to_string();
            tokio::spawn(async move {
                println!("Spawning thread to play game...");
                play_game(&cloned_token, &lichess_game_full.id, &lichess_game_full.fen).await;
            });

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

async fn accept_challenge(token: &str, game_id: &str) -> Result<(), String> {
    let lichess_url = format!("https://lichess.org/api/challenge/{game_id}/accept");
    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .post(lichess_url)
        .bearer_auth(token)
        .send()
        .await;

    let parsed_response = match response_result {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    if parsed_response.status() != 200 {
        return Err(format!("Something went wrong trying to accept challenge.\nAPI response: {:#?}.", parsed_response.text().await));
    }

    return Ok(());
}

async fn write_chat_message(token: &str, game_id: &str, message: &str) -> Result<(), String> {
    let lichess_url = format!("https://lichess.org/api/bot/game/{game_id}/chat");
    let mut params = HashMap::new();
    params.insert("room", "player"); // player/spectator
    params.insert("text", message);

    let client: reqwest::Client = reqwest::Client::new();
    let response_result: Result<reqwest::Response, reqwest::Error> = client
        .post(lichess_url)
        .bearer_auth(token)
        .form(&params)
        .send()
        .await;

    let parsed_response = match response_result {
        Ok(r) => r,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    if parsed_response.status() != 200 {
        return Err(format!("Something went wrong trying to write chat message.\nAPI response: {:#?}.", parsed_response.text().await));
    }

    return Ok(());
}
