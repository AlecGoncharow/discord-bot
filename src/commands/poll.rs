use serenity::model::id::ChannelId;
use serenity::utils::MessageBuilder;
use std::collections::HashMap;

lazy_static! {
    static ref LOOKUP: HashMap<u32, String> = {
        let mut lookup = HashMap::new();
        let vec = vec![
            '\u{30}', '\u{31}', '\u{32}', '\u{33}', '\u{34}', '\u{35}', '\u{36}', '\u{37}',
            '\u{38}', '\u{39}',
        ];
        for i in 0..10 {
            let mut emoji = String::new();
            emoji.push(vec[i]);
            emoji.push('\u{20E3}');
            lookup.insert(i as u32, emoji);
        }

        lookup
    };
}

command!(poll(_ctx, msg, msg_args) {
    let channel_id = ChannelId(519024472930648065);
    let args: Vec<&str> = msg_args.full().split(";").collect();
    let title = args[0];
    let is_help = title.len() == 0;
    if is_help {
        let mut content = MessageBuilder::new()
            .push("```md\n# Options:\n1. -poll yes or no question here\n")
            .push("2. -poll question; options, delimited by, commas\n```")
            .build();
        let _ = msg.reply(&content);
    } else {
        let mut content = MessageBuilder::new()
            .push("```md\n#")
            .push(&msg.author.name)
            .push("'s New Patch btw Poll: ")
            .push(&title);
        let mut has_args = args.len() > 1;
        let mut num = 0;
        if has_args {
            let options: Vec<&str> = args[1].split(",").collect();
            for item in options {
                content = content
                    .push("\n")
                    .push(num)
                    .push(". ")
                    .push(item);
                num += 1;
            }
        }
        content = content.push("```");
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
            for i in 0..num {
                let resp = LOOKUP.get(&i).unwrap().to_owned();
                let _ = res.react(resp);
            }
        }
    }
});
