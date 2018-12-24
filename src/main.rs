#[macro_use]
extern crate serenity;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate artifact_lib;
extern crate chrono;
extern crate rand;
extern crate regex;
extern crate serde_json;

use artifact_lib::Artifact;
use serenity::{framework::StandardFramework, model::gateway::Ready, prelude::*};
use std::env;

mod commands;
struct Handler;

lazy_static! {
    static ref ARTIFACT: Artifact = Artifact::new();
}

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("{:?}", ARTIFACT.card_from_name("drow ranger").unwrap());
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
            .cmd("tips", commands::tip::tip_log)
            .cmd("tip", commands::tip::tip)
            .cmd("antitip", commands::tip::tip)
            .cmd("profile", commands::tip::profile)
            .cmd("card", commands::artifact::card)
            .group("Artifact", |g| {
                g.prefix("artifact")
                    .command("card", |c| c.cmd(commands::artifact::card))
            }),
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}
