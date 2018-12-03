#[macro_use]
extern crate serenity;

use serenity::client::Client;
use serenity::framework::standard::StandardFramework;
use serenity::prelude::EventHandler;
use std::env;

struct Handler;

impl EventHandler for Handler {}
fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_BOT_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("-")) // set the bot's prefix to "~"
            .cmd("ping", ping),
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});

command!(mort(_context, message) {
    let _ = message.reply("Don't Believe His Lies");
});
