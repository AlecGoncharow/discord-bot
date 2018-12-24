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
    } else if named_card.gold_cost != 0{
        Colour::GOLD
    } else {
        Colour::LIGHT_GREY
    };

    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title(&named_card.card_name.english)
                                               .description(&desc)
                                               .color(color)
                                               .image(&named_card.large_image.default)
                                               ));
}

fn get_references(card: &artifact_lib::Card, refs:&mut Vec<u32>) {
    for r in &card.references {
        if !refs.contains(&r.card_id) {
            refs.push(r.card_id);   
            let child = crate::ARTIFACT.card_from_id(r.card_id).unwrap();
            get_references(&child, refs);
        }
    }
}

command!(get_card(_ctx, msg, msg_args) {
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
                let is_single = match v {
                    artifact_lib::NamedCard::Single(_) => true,
                    _ => false,
                };
                if is_single {
                    let value = match v {
                        artifact_lib::NamedCard::Single(s) => s,
                        _ => panic!("something terrible has happened"),
                    };
                    print_card(msg, value);
                    count += 1;
                } else {
                    let value = match v {
                        artifact_lib::NamedCard::Multiple(m) => m,
                        _ => panic!("something terrible has happened"),
                    };
                    for c in value {
                        print_card(msg, c);
                        count += 1;
                    }
                }
            }
        } else {
            let named_card_wrapped: &artifact_lib::NamedCard = lookup.unwrap();
            let is_single = match named_card_wrapped {
                artifact_lib::NamedCard::Single(_) => true,
                _ => false,
            };
 
            if is_single {
                let named_card = match named_card_wrapped { 
                    artifact_lib::NamedCard::Single(s) => s,
                    _ => panic!("something terrible has happened"),
                };
                print_card(msg, named_card);
                let mut refs = Vec::new();
                get_references(&named_card, &mut refs);
                for r in refs {
                    let card = crate::ARTIFACT.card_from_id(r).unwrap();
                    print_card(msg, card);
                }

            } else {
                let value = match named_card_wrapped {
                    artifact_lib::NamedCard::Multiple(m) => m,
                    _ => panic!("something terrible has happened"),
                };
                let mut refs = Vec::new();
                for c in value {
                    if !refs.contains(&c.card_id) {
                        refs.push(c.card_id);
                    }
                    get_references(&c, &mut refs);
                }   
                for r in &refs {
                    let card = crate::ARTIFACT.card_from_id(*r).unwrap();
                    print_card(msg, card);
                }
            }
        }
    }
});
