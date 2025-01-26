# What is this

This is a chess engine! I enjoy chess, and wanted to learn rust. I figured a good way to learn was to write an engine!

This chess engine uses bitboards, alpha-beta pruning, zobrist-hashes, lookup tables, iterative deepening, and many more features. It also interacts with the Lichess Bot API; so the engine can be played against on their site with their UI.

Huge shout out to [The Chess Programming Wiki](https://www.chessprogramming.org/Main_Page), for ideas and approaches to problems encountered when building the engine. I also had help from [Chess Programming's YouTube Channel](https://www.youtube.com/@chessprogramming591); especially for some of the magic bitboard stuff. That part is a bit over my head.

Also big shout out to [Sebastian Lague's YouTube channel](https://www.youtube.com/@SebastianLague) for inspiring me to make this. He makes great videos.

# How to run

1. Clone this repo.
1. Create a new Lichess Account, and upgrade it to a bot account. Details can be found [here](https://lichess.org/api#tag/Bot/operation/apiBotOnline), and make sure you save your auth token.
1. Add your bot's token as an environment variable named `LICHESS_BOT_API_TOKEN`.
1. Update `constants.rs`, the code needs your bot's username; and a whitelist of challenger usernames. Variables are called `LICHESS_BOT_USERNAME` and `LICHESS_CHALLENGER_WHITELIST`. Sorry, this should have been in a config file. I may fix it later.
1. At this point, you should be able to run your bot. Just use `cargo run` in the directory for this repo.
1. The bot should be up and running, now you may issue a challenge to it. Then you can play against it!

# Future improvements

-   Opening weakness. Skilled players can get an advantage out of the opening. Add an opening book?
-   The engine seems to struggle with some endgames, even up a lot of material.
-   Move ordering with iterative deepening? Also timing on how long to iteratively deepen.
-   Tweak the size of our transposition table. There is surely some fine-tuning that can be done there.
-   Create more happy king squares for endgame specifically (king wants to be in different spots in early vs endgame).
