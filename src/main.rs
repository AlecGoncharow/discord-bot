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
            .cmd("morton", mort)
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

use serenity::model::id::ChannelId;
use serenity::utils::MessageBuilder;
command!(poll(_ctx, msg, args) {
    let channel_id = ChannelId(519024472930648065);
    let mut valid = true;
    let title = match args.single_quoted::<String>()  {
        Ok (t) => t,
        Err(_) => {
            valid = false;
            String::from("You must give at least one quoted argument ex: -poll \"I know how to make poll\"")
            },
    };
    if valid {
        let mut content = MessageBuilder::new()
            .push("Poll: ")
            .push(&title);

        let mut has_args = false;
        let mut num = 0;
        loop {
            let a = match args.single_quoted::<String>() {
                Ok(t) => t,
                Err(_) => break,
            };
            has_args = true;
            content = content
                .push("\n")
                .push(num)
                .push(": ")
                .push(a);
            num += 1;
        }
        let res = match channel_id.say(&content) {
            Ok(p) => p,
            Err(e) => panic!("error sending message: {}", e),
        };
        if !has_args {
            let _ = match res.react('\u{1F44E}'){
                Ok(s) => s,
                Err(e) => panic!("error reacting: {}", e),
            };
            let _ = res.react('\u{1F44D}');
        } else {
            // There has to be a better way...
            let mut lookup = std::collections::HashMap::new();
            lookup.insert(0, '\u{30}');
            lookup.insert(1, '\u{31}');
            lookup.insert(2, '\u{32}');
            lookup.insert(3, '\u{33}');
            lookup.insert(4, '\u{34}');
            lookup.insert(5, '\u{35}');
            lookup.insert(6, '\u{36}');
            lookup.insert(7, '\u{37}');
            lookup.insert(8, '\u{38}');
            lookup.insert(9, '\u{39}');
            for i in 0..num {
                let unicode = lookup.get(&i).unwrap().to_owned();
                let a = '\u{20E3}';
                let mut resp = String::from("");
                resp.push(unicode);
                resp.push(a);
                let _ = res.react(resp);
            }
        }
    }
    else {
        let _ = msg.reply(&title);
    }
});
