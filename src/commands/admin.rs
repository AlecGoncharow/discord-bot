command!(quit(ctx, msg, _args) {
    ctx.quit();

    let _ = msg.reply("Shutting down!");
    std::process::exit(0);
});