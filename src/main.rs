#[macro_use]
extern crate serenity;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate serde_json;

use serenity::{framework::StandardFramework, model::gateway::Ready, prelude::*};
use std::env;

mod commands;
struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_BOT_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("-")) // set the bot's prefix to "-"
            .cmd("ping", commands::misc::ping)
            .cmd("mort", commands::misc::mort)
            .cmd("morton", commands::misc::mort)
            .cmd("multiply", commands::misc::multiply)
            .cmd("poll", commands::poll::poll)
            .cmd("random", commands::dota::random)
            .cmd("tip", commands::tip::tip),
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}
