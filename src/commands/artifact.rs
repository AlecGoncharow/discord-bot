use regex::Regex;
use serenity::utils::Colour;

fn print_card(msg: &serenity::model::channel::Message, named_card: &artifact_lib::Card) {
    let line_break = Regex::new(r"<br/>").unwrap();
    let html = Regex::new(r"<[^<]*>").unwrap();

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
    let mut refs = Vec::new();
    get_references(named_card, &mut refs);
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for r in refs {
        let current: &artifact_lib::Card = crate::ARTIFACT.card_from_id(r).unwrap();
        let text = current.card_text.english.clone();
        let break_lines = line_break.replace_all(&text, "\n");
        let desc = html.replace_all(&break_lines, "");
        let info = if current.card_type == "Creep" {
            format!("Health: {}, Armor: {}, Damage: {}", current.hit_points, current.armor, current.attack)
        } else if current.card_type == "Spell" {
            format!("Mana Cost: {}", current.mana_cost)
        } else if current.card_type == "Item" {
            format!("Gold Cost: {}", current.gold_cost)
        } else {
            String::from("")
        };
        fields.push((
                format!("{}: {}", current.card_type, current.card_name.english),
                format!("{}\n{}", desc, info),
                false
        ));
    }

    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title(&named_card.card_name.english)
                                               .color(color)
                                               .fields(fields)
                                               .thumbnail(&named_card.mini_image.default)
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
            let mut matches = crate::ARTIFACT.name_map.iter().filter(|&(k, _)| k.contains(&card_name.to_lowercase()));
            let count = matches.clone().count();
            let line_break = Regex::new(r"<br/>").unwrap();
            let html = Regex::new(r"<[^<]*>").unwrap();

            if count == 0 {
                let _ = msg.reply(&format!("{} is not a valid card name, substring search found nothing"
                                           , card_name));
            }
            else if count == 1 {
                let (_, named_card_wrapped) = matches.next().unwrap();

                if named_card_wrapped.is_single() {
                    let named_card = match named_card_wrapped { 
                        artifact_lib::NamedCard::Single(s) => s,
                        _ => panic!("something terrible has happened"),
                    };
                    print_card(msg, named_card);
                } else {
                    let value = match named_card_wrapped {
                        artifact_lib::NamedCard::Multiple(m) => m,
                        _ => panic!("something terrible has happened"),
                    };
                    for card in value {
                        if card.large_image.default != "" {
                            print_card(msg, &card);
                            break;
                        }
                    }
                }
            } else {
                let mut fields = Vec::new();
                let mut count = 0;
                for (_, card) in matches {
                    if count > 10 {
                        break;
                    }

                    if card.is_single() {
                        let named_card = match card { 
                            artifact_lib::NamedCard::Single(s) => s,
                            _ => panic!("something terrible has happened"),
                        };
                        let text = named_card.card_text.english.clone();
                        let break_lines = line_break.replace_all(&text, "\n");
                        let desc = html.replace_all(&break_lines, "");
                        fields.push(
                            (
                                format!("{}: {}", named_card.card_type, named_card.card_name.english),
                                format!("{}", desc),
                                true
                            )
                        );

                    } else {
                        let value = match card {
                            artifact_lib::NamedCard::Multiple(m) => m,
                            _ => panic!("something terrible has happened"),
                        };
                        for named_card in value {
                            if named_card.large_image.default != "" {
                                let text = named_card.card_text.english.clone();
                                let break_lines = line_break.replace_all(&text, "\n");
                                let desc = html.replace_all(&break_lines, "");
                                fields.push(
                                    (
                                        format!("{}: {}", named_card.card_type, named_card.card_name.english),
                                        format!("{}", desc),
                                        false
                                    )
                                );
                                break;
                            }
                    }
                }

                    count += 1;
                }
                let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title(&format!("Cards matching {}:"
                                                               , card_name))
                                               .fields(fields)
                                               ));
            }
        } else {
            let named_card_wrapped: &artifact_lib::NamedCard = lookup.unwrap();

            if named_card_wrapped.is_single() {
                let named_card = match named_card_wrapped { 
                    artifact_lib::NamedCard::Single(s) => s,
                    _ => panic!("something terrible has happened"),
                };
                print_card(msg, named_card);
            } else {
                let value = match named_card_wrapped {
                    artifact_lib::NamedCard::Multiple(m) => m,
                    _ => panic!("something terrible has happened"),
                };
                for card in value {
                    if card.large_image.default != "" {
                        print_card(msg, &card);
                        break;
                    }
                }
            }
        }
    }
});
