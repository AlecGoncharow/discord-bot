use regex::Regex;

command!(card(_ctx, msg, msg_args) {
    let card_name = msg_args.full();
    if card_name.len() == 0 {
        let _ = msg.reply("pass a card name to get the card's image back");
    } else {
        let lookup = crate::ARTIFACT.card_from_name(card_name);
        if lookup.is_none() {
            let _ = msg.reply(&format!("{} is not a valid card name", card_name));
        } else {
            let line_break = Regex::new(r"<br/>").unwrap();
            let html = Regex::new(r"<[^<]*>").unwrap();

            let named_card: &artifact_lib::Card = lookup.unwrap();
            let text = named_card.card_text.english.clone();
            let break_lines = line_break.replace_all(&text, "\n");
            let desc = html.replace_all(&break_lines, "");
            let _ = msg.channel_id.send_message(|m| m
                                                .embed(|e| e
                                                       .title(&named_card.card_name.english)
                                                       .description(&desc)
                                                       .image(&named_card.large_image.default)));
        }
    }
});
