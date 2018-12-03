#[macro_use]
extern crate serenity;
use serenity::{framework::StandardFramework, model::gateway::Ready, prelude::*};
use std::env;

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
            .cmd("ping", ping)
            .cmd("mort", mort)
            .cmd("multiply", multiply)
            .cmd("poll", poll),
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
    let _ = message.channel_id.say("Don't Believe His Lies");
});

command!(multiply(_ctx, msg, args) {
    let mut product = 1 as f32;
    loop {
        let a = match args.single::<f32>() {
            Ok(r) => r,
            Err(_) => break,
        };
        product *= a;
    }
    let _ = msg.reply(&product.to_string());
});

command!(poll(_ctx, msg, args) {
    let re = args.single_quoted::<String>().unwrap();
    let _ = msg.reply(&re);
});
