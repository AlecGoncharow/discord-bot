use rand::prelude::*;
use serenity::utils::MessageBuilder;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Hero {
    localized_name: String,
    url_full_portrait: String,
    name: String,
    url_small_portrait: String,
    url_large_portrait: String,
    url_vertical_portrait: String,
    id: u8,
}

#[derive(Serialize, Deserialize)]
struct Heroes {
    status: u8,
    count: u8,
    heroes: Vec<Hero>,
}

command!(random(_ctx, msg) {
    let path = Path::new("./data/dota/heroes.json");
    let json = File::open(path).expect("file not found");
    let data: Heroes = serde_json::from_reader(json).expect("Error while reading json");
    let heroes = data.heroes;

    let mut rng = rand::thread_rng();
    let y: u8 = rng.gen_range(0, data.count);
    let hero = heroes.get(y as usize).unwrap();

    let mut content = MessageBuilder::new()
                            .push(&hero.url_full_portrait)
                            .push("\n```md\n#####\t")
                            .push(&hero.localized_name)
                            .push("\t#####\n```")
                            .build();
    let _ = msg.reply(&content);
});
