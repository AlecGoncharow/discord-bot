use chrono::{Local, TimeZone};
use serenity::utils::MessageBuilder;
use std::fs::OpenOptions;
use std::path::Path;

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_WEEK: u64 = 604800;
const WEEKLY_TIPS: u32 = 7;
const WEEKLY_ANTI_TIPS: u32 = 1;
const CHANNEL: u64 = 527728791876009994;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    lifetime_gross: i32,
    lifetime_net: i32,
    week_gross: i8,
    week_net: i8,
    tips: u32,
    tips_given: u32,
    anti_tips: u32,
    anti_tips_given: u32,
}

impl Default for User {
    fn default() -> User {
        User {
            id: 0,
            lifetime_gross: 0,
            lifetime_net: 0,
            week_gross: 0,
            week_net: 0,
            tips: WEEKLY_TIPS,
            tips_given: 0,
            anti_tips: WEEKLY_ANTI_TIPS,
            anti_tips_given: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Data {
    reset_time: u64,
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Tip {
    tipper_id: u64,
    tipper_name: String,
    tipee_id: u64,
    tipee_name: String,
    time: u64,
    is_anti: bool,
}

#[derive(Serialize, Deserialize)]
struct Tips {
    tips: Vec<Tip>,
}

command!(profile(_ctx, msg, msg_args) {
    let is_other = match msg_args.single_n::<serenity::model::id::UserId>() {
        Ok(_) => true,
        Err(_) => false,
    };

    let user: serenity::model::user::User = if is_other {
        msg_args.single::<serenity::model::id::UserId>().unwrap().to_user().unwrap()
    } else {
        msg.author.clone()
    };

    let key = std::env::var("ALECA_KEY").unwrap();
    let tip_user: User = get_user(*user.id.as_u64(), "https://aleca-api.herokuapp.com", &key);
    let avatar = user.face();
    let ava_url = if avatar.ends_with(".webp?size=1024") {
        &avatar[..avatar.len() - 15]
    } else {
        avatar.as_str()
    };

    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title("Tip Profile")
                                               .thumbnail(
                                                   ava_url
                                                   )
                                               .field("Tips Recieved: All Time",
                                                      tip_user.lifetime_net,
                                                      true)
                                               .field("Tips Recieved: This Week",
                                                      tip_user.week_net,
                                                      true)
                                               .field("Tips Given: All Time",
                                                      tip_user.tips_given,
                                                      false)
                                               .field("Weekly Tips Remaining",
                                                      tip_user.tips,
                                                      true)
                                               .field("Anti Tips Given: All Time",
                                                      tip_user.anti_tips_given,
                                                      true)
                                               .field("Weekly Anti Tips Remaining",
                                                      tip_user.anti_tips,
                                                      true)
                                               .color(
                                                    serenity::utils::Colour::GOLD
                                                )
                                               ));

});

enum TipResult {
    Ok,
    SameId,
    NoTips,
    WriteErr(String),
}

enum TipAction {
    Give(bool),
    Receive(bool),
}

command!(handle_tip(_ctx, msg, msg_args) {
    let is_anti = msg.content.starts_with("-anti");
    let url = "https://aleca-api.herokuapp.com";
    let key = std::env::var("ALECA_KEY").unwrap();
    let is_tip = match msg_args.single_n::<serenity::model::id::UserId>() {
        Ok(_) => true,
        Err(_) => false,
    };

    if is_tip {
        let tipper_id = msg.author.id;
        let tipee_id = match msg_args.single_n::<serenity::model::id::UserId>() {
            Ok(id) => id.clone(),
            _ => panic!("Something terrible has happened"),
        };
        let success = match transact_tip(*tipper_id.as_u64(), *tipee_id.as_u64(), is_anti, url, &key) {
            TipResult::SameId => {
                if is_anti {
                    let _ = msg.reply("https://www.youtube.com/watch?v=9Deg7VrpHbM");
                }
                else {
                    let _ = msg.reply("you can't tip yourself lol");
                }
                false
            }
            TipResult::NoTips => {
                let _ = msg.reply("you are out of that kind of tip for this week");
                false
            }
            TipResult::Ok => true,
            _ => false,
        };
        send_response(*tipper_id.as_u64(), *tipee_id.as_u64(), url, is_anti);
    }
});

fn send_response(from: u64, to: u64, url: &str, is_anti: bool) {
    let channel_id = serenity::model::id::ChannelId(CHANNEL);
    let tipper = get_user(from, url, "");
    let tipee = get_user(to, url, "");
    let tipee_disc = serenity::model::id::UserId(to).to_user().unwrap();
    let tipper_disc = serenity::model::id::UserId(from).to_user().unwrap();
    let avatar = tipee_disc.face();
    let ava_url = if avatar.ends_with(".webp?size=1024") {
        &avatar[..avatar.len() - 15]
    } else {
        avatar.as_str()
    };

    let tip_text = if is_anti { "anti-tipped" } else { "tipped" };
    let giver_title = if is_anti {
        format!("{} Anti-Tips Given", tipper_disc.name)
    } else {
        format!("{} Tips Given", tipper_disc.name)
    };

    let giver_desc = if is_anti {
        format!(
            "lifetime anti-tips given: {}\nweekly anti-tips left to give: {}",
            tipper.anti_tips_given, tipper.anti_tips
        )
    } else {
        format!(
            "lifetime tips given: {}\nweekly tips left to give: {}",
            tipper.tips_given, tipper.tips
        )
    };
    let _ = channel_id.send_message(|m| {
        m.embed(|e| {
            e.title(format!(
                "{} {} {}",
                tipper_disc.name, tip_text, tipee_disc.name
            ))
            .thumbnail(ava_url)
            .field(giver_title, giver_desc, false)
            .field(
                format!("{} Tips Recieved", tipee_disc.name),
                format!(
                    "lifetime net: {}\n lifetime gross: {}\nweek net: {}\nweek gross: {}",
                    tipee.lifetime_net, tipee.lifetime_gross, tipee.week_net, tipee.week_gross
                ),
                false,
            )
            .color(serenity::utils::Colour::GOLD)
        })
    });
}

fn transact_tip(tipper_id: u64, tipee_id: u64, is_anti: bool, url: &str, key: &str) -> TipResult {
    if tipper_id == tipee_id {
        return TipResult::SameId;
    }

    let tipper_user = get_user(tipper_id, url, key);
    let tipee_user = get_user(tipee_id, url, key);
    if is_anti {
        if tipper_user.anti_tips == 0 {
            return TipResult::NoTips;
        }
        reqwest::get(
            format!(
                "{}/tips/anti_tip/{}/{}?key={}",
                url, tipper_id, tipee_id, key
            )
            .as_str(),
        );
    } else {
        if tipper_user.tips == 0 {
            return TipResult::NoTips;
        }
        reqwest::get(format!("{}/tips/tip/{}/{}?key={}", url, tipper_id, tipee_id, key).as_str());
    }

    TipResult::Ok
}

fn create_user(id: u64, url: &str, key: &str) -> User {
    let mut user = User::default();
    user.id = id;

    reqwest::get(format!("{}/tips/create/{}?key={}", url, id, key).as_str());

    get_user(id, url, key)
}

fn get_user(id: u64, url: &str, key: &str) -> User {
    let go = format!("{}/tips/{}", url, id);
    println!("{}", go);

    let mut req = reqwest::get(format!("{}/tips/{}", url, id).as_str()).unwrap();
    println!("{:?}", req);
    if req.status() == reqwest::StatusCode::NOT_FOUND {
        create_user(id, url, key)
    } else {
        req.json().unwrap()
    }
}
