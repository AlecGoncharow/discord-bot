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
