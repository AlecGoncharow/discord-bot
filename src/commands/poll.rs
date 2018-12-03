use serenity::model::id::ChannelId;
use serenity::utils::MessageBuilder;
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
            .push("'s Poll: ")
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
});
