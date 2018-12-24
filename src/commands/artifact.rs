use regex::Regex;
use serenity::utils::Colour;

fn print_card(msg: &serenity::model::channel::Message, named_card: &artifact_lib::Card) {
    let line_break = Regex::new(r"<br/>").unwrap();
    let html = Regex::new(r"<[^<]*>").unwrap();

    let text = named_card.card_text.english.clone();
    let break_lines = line_break.replace_all(&text, "\n");
    let desc = html.replace_all(&break_lines, "");
    let color = if named_card.is_red {
        Colour::DARK_RED
    } else if named_card.is_green {
        Colour::DARK_GREEN
    } else if named_card.is_blue {
        Colour::DARK_BLUE
    } else if named_card.is_black { 
        Colour::DARK_PURPLE
    } else {
        Colour::GOLD
    }; 

    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title(&named_card.card_name.english)
                                               .description(&desc)
                                               .color(color)
                                               .image(&named_card.large_image.default)));
}


command!(card(_ctx, msg, msg_args) {
    let card_name = msg_args.full();
    if card_name.len() == 0 {
        let _ = msg.reply("pass a card name to get the card's image back");
    } else {
        let lookup = crate::ARTIFACT.card_from_name(card_name);
        if lookup.is_none() {
            let _ = msg.reply(&format!("{} is not a valid card name, starting substring search"
                                       , card_name));

            let matches = crate::ARTIFACT.name_map.iter().filter(|&(k, _)| k.contains(&card_name.to_lowercase()));
            let mut count = 0;
            for (_, v) in matches {
                if count > 4 {
                    let _ = msg.reply("more than 5 cards were found, stopping");
                    break;
                }
                print_card(msg, v);
                count += 1;
            }
        } else {
            let named_card: &artifact_lib::Card = lookup.unwrap();
 
            print_card(msg, named_card);
        }
    }
});
