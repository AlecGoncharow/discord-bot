use regex::Regex;
use serenity::utils::Colour;

fn print_card(msg: &serenity::model::channel::Message, named_card: &artifact_lib::Card) {
    let color = if named_card.is_red {
        Colour::DARK_RED
    } else if named_card.is_green {
        Colour::DARK_GREEN
    } else if named_card.is_blue {
        Colour::DARK_BLUE
    } else if named_card.is_black {
        Colour::DARK_PURPLE
    } else if named_card.gold_cost != 0 {
        Colour::GOLD
    } else {
        Colour::LIGHT_GREY
    };
    let mut refs = Vec::new();
    get_references(named_card, &mut refs);
    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for r in refs {
        let current: &artifact_lib::Card = crate::ARTIFACT.card_from_id(r).unwrap();
        let desc = get_description(&current);
        let info = get_info(&current, &named_card);
        fields.push((
            format!("{}: {}", current.card_type, current.card_name.english),
            format!("{}\n{}", desc, info),
            false,
        ));
    }

    let _ = msg.channel_id.send_message(|m| {
        m.embed(|e| {
            e.title(&named_card.card_name.english)
                .color(color)
                .fields(fields)
                .thumbnail(&named_card.mini_image.default)
                .image(&named_card.large_image.default)
        })
    });
}

fn get_description(named_card: &artifact_lib::Card) -> String {
    let line_break = Regex::new(r"<br/>").unwrap();
    let html = Regex::new(r"<[^<]*>").unwrap();
    let text = named_card.card_text.english.clone();
    let break_lines = line_break.replace_all(&text, "\n");
    String::from(html.replace_all(&break_lines, ""))
}

fn get_info(named_card: &artifact_lib::Card, parent: &artifact_lib::Card) -> String {
    match named_card.card_type.as_str() {
        "Hero" => format!(
            "Health: {}, Armor: {}, Damage: {}",
            named_card.hit_points, named_card.armor, named_card.attack
        ),
        "Creep" => format!(
            "Mana Cost: {}, Health: {}, Armor: {}, Damage: {}",
            named_card.mana_cost, named_card.hit_points, named_card.armor, named_card.attack
        ),
        "Spell" => format!("Mana Cost: {}", named_card.mana_cost),
        "Item" => format!(
            "Gold Cost: {}, Type: {}",
            named_card.gold_cost, named_card.sub_type
        ),
        "Ability" => {
            let cooldown = Regex::new(r"Active (\d+)").unwrap();
            match cooldown.captures(&parent.card_text.english) {
                Some(m) => {
                    let cd = m.get(1).unwrap().as_str();
                    format!("Cooldown: {}", cd)
                }
                None => String::from(""),
            }
        }
        _ => String::from(""),
    }
}

fn get_references(card: &artifact_lib::Card, refs: &mut Vec<u32>) {
    for r in &card.references {
        if !refs.contains(&r.card_id) {
            refs.push(r.card_id);
            let child = crate::ARTIFACT.card_from_id(r.card_id).unwrap();
            get_references(&child, refs);
        }
    }
}

command!(get_deck(_ctx, msg, _msg_args) {
    let adc = Regex::new(r"(ADC[^\n]+)").unwrap();
    let caps = adc.captures(&msg.content);
    if caps.is_some() {
        let deck_code = caps.unwrap().get(1).unwrap().as_str();
        let mut deck = crate::ARTIFACT.get_deck(deck_code).unwrap();
        deck.heroes.sort();
        deck.cards.sort();
        println!("{:?}", deck);
        /*
           let image_url = format!("https://www.playartifact.com/thumbnail/{}_{}_{}_{}_{}",
           deck.heroes[0].card.card_id,
           deck.heroes[1].card.card_id,
           deck.heroes[2].card.card_id,
           deck.heroes[3].card.card_id,
           deck.heroes[4].card.card_id,
           );

*/
        let mut fields: Vec<(String, String, bool)> = Vec::new();

        for hero in &deck.heroes {
            let title = format!("{}: {}", hero.turn, hero.card.card_name.english.clone());
            let desc = get_info(&hero.card, &hero.card);
            println!("{}", title);
            fields.push((
                    title,
                    format!("{}", desc),
                    false,
            ));
        }

        for card in &deck.cards {
            let title = format!("{} ({})", card.card.card_name.english, card.color );
            let desc = get_info(&card.card, &card.card);
            fields.push((
                    title,
                    format!("{}", desc),
                    false,
            ));
        }
        let name = deck.name.clone();

        println!("{:?}", fields);

        let res = msg.channel_id.send_message(|m| m
                                              .embed(|e| e
                                                     .title(&name)
                                                     .fields(fields)
                                              ));
        println!("{:?}",res);
    }
});

command!(get_card(_ctx, msg, msg_args) {
    let card_name = msg_args.full();
    if card_name.len() == 0 {
        let _ = msg.reply("pass a card name to get the card's image back");
    } else {
        let lookup = crate::ARTIFACT.card_from_name(card_name);
        if lookup.is_none() {
            let mut matches = crate::ARTIFACT.name_map.iter().filter(|&(k, _)| k.contains(&card_name.to_lowercase()));
            let count = matches.clone().count();

            if count == 0 {
                let _ = msg.reply(&format!("{} is not a valid card name, substring search found nothing"
                                           , card_name));
            }
            else if count == 1 {
                let (_, named_card_wrapped) = matches.next().unwrap();

                let named_card = named_card_wrapped.clone().into_card().unwrap();
                print_card(msg, &named_card);

            } else {
                let mut fields = Vec::new();
                let mut count = 0;
                for (_, card) in matches {
                    if count > 10 {
                        break;
                    }

                    let named_card = card.clone().into_card().unwrap();

                    let desc = get_description(&named_card);
                    fields.push(
                        (
                            format!("{}: {}", named_card.card_type, named_card.card_name.english),
                            format!("{}", desc),
                            false
                        )
                    );

                    count += 1;
                }
                let res = msg.channel_id.send_message(|m| m
                                                      .embed(|e| e
                                                             .title(&format!("Cards matching {}:"
                                                                             , card_name))
                                                             .fields(fields)
                                                      ));
                match res {
                    Err(_) => {let _ = msg.reply("Too vague of a query, try to be more specific");}
                    Ok(_) => (),
                }
            }
        } else {
            let named_card_wrapped: &artifact_lib::NamedCard = lookup.unwrap();

            let named_card = named_card_wrapped.clone().into_card().unwrap();
            print_card(msg, &named_card);
        }
    }
});
