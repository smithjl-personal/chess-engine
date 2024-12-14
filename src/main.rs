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
use constants::INITIAL_GAME_STATE_FEN;
use my_move::Move;
use serde_json;
use std::env;

#[tokio::main]
async fn main() {
    play_lichess_game(String::from("AONDtwjfurtG")).await;
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
            println!("Opponent talked in chat. Code not set up to handle this.");
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
                    Some(m) => game.make_move(&m),
                    None => {
                        println!("Tried to make 'illegal' move...");
                        //self.print_all_legal_moves();
                        continue;
                    }
                }

                //let internal_move

                //game.make_move(&m, Some(true));
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

            // Handle errors later...
            let last_move = Move::str_to_move(&last_move_str);
            game.make_move(&last_move.unwrap());

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

        //println!("Chunk: {chunk:?}, Length: {}", chunk.len());
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
