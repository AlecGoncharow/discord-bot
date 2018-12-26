use serenity::utils::MessageBuilder;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;
use chrono::{Local, TimeZone};

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_WEEK: u64 = 604800;
const WEEKLY_TIPS: u8 = 7;
const WEEKLY_ANTI_TIPS: u8 = 1;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    user_id: u64,
    lifetime_gross: i32,
    lifetime_net: i32,
    week_gross: i8,
    week_net: i8,
    tips: u8,
    tips_given: u32,
    anti_tips: u8,
    anti_tips_given: u32,
}

impl Default for User {
    fn default() -> User {
        User {
            user_id: 0,
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

command!(tip_log(_ctx, msg) {
    let mut tip_data: Tips = get_log();

    let mut out: Vec<Tip> = Vec::new();
    let len = tip_data.tips.len();
    if len > 5 {
        for i in 1..=5 {
            out.push(tip_data.tips[len - i].clone()); 
        }
    } else {
        out = tip_data.tips;
        out.reverse();
    }

    let mut content = MessageBuilder::new()
            .push("```md\n")
            .push("### Most Recent Tips ###");

    for entry in out {
        let tip_text = if entry.is_anti {
            "anti tipped"
        } else {
            "tipped"
        };
        content = content.push(format!("\n* [{}] {} {} {}", Local.timestamp(entry.time as i64,0), 
                             entry.tipper_name, 
                             tip_text,
                             entry.tipee_name));
    }

    let resp = content.push("\n```").build();
    let _ = msg.channel_id.say(&resp);
});

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

    let tip_user: User = get_user(*user.id.as_u64());
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
    WriteErr,
}

enum TipAction {
    Give(bool),
    Recieve(bool),
}

command!(handle_tip(_ctx, msg, msg_args) {
    match check_tips() {
        TipResult::Ok => println!("tips up to date"),
        _ => panic!("something terrible has happened"),
    }
    let is_anti = msg.content.starts_with("-anti");
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
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let success = match transact_tip(*tipper_id.as_u64(), *tipee_id.as_u64(), is_anti) {
            TipResult::SameId => {
                let _ = msg.reply("you can't tip yourself lol");
                false
            }
            TipResult::Ok => true,
            _ => false,
        };
        if success {
            let mut tips_log = get_log();
            let tipee_user = tipee_id.to_user().unwrap();
            let tipper_user = tipper_id.to_user().unwrap();
            let tip = Tip {
                tipper_id: *tipper_id.as_u64(),
                tipper_name: tipper_user.name,
                tipee_id: *tipee_id.as_u64(),
                tipee_name: tipee_user.name,
                time: time.as_secs(),
                is_anti,
            };

            send_response(&tip, msg);
            tips_log.tips.push(tip);
            write_log(&tips_log);
        }
    }
});

fn send_response(tip: &Tip, msg: &serenity::model::channel::Message) {
    let data = get_data();
    let tipper = get_user(tip.tipper_id);
    let tipee = get_user(tip.tipee_id);
    let tipee_disc = serenity::model::id::UserId(tip.tipee_id);
    let avatar = tipee_disc.to_user().unwrap().face();
    let ava_url = if avatar.ends_with(".webp?size=1024") {
        &avatar[..avatar.len() - 15]
    } else {
        avatar.as_str()
    };
    let tip_text = if tip.is_anti {
        "anti-tipped"
    } else {
        "tipped"
    };

    let giver_title = if tip.is_anti {
        format!("{} Anti-Tips Given", tip.tipper_name)
    } else {
        format!("{} Tips Given", tip.tipper_name)
    };

    let giver_desc = if tip.is_anti {
        format!("lifetime anti-tips given: {}\nweekly anti-tips left to give: {}", tipper.anti_tips_given, tipper.anti_tips)  
    } else {
        format!("lifetime tips given: {}\nweekly tips left to give: {}", tipper.tips_given, tipper.tips)  
    };
    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title(format!("{} {} {}", tip.tipper_name,
                                                              tip_text,
                                                              tip.tipee_name))
                                               .thumbnail(
                                                   ava_url
                                                   )
                                               .field(giver_title,
                                                      giver_desc,
                                                      false)
                                               .field(format!("{} Tips Recieved", tip.tipee_name),
                                                      format!("lifetime net: {}\n lifetime gross: {}\nweek net: {}\nweek gross: {}", tipee.lifetime_net, tipee.lifetime_gross, tipee.week_net, tipee.week_gross),
                                                      false)
                                               .field("Weekly Reset Time",
                                                      Local.timestamp(data.reset_time as i64, 0 ),
                                                      false
                                                      )
                                               .color(
                                                    serenity::utils::Colour::GOLD
                                                )
                                               ));

}

fn transact_tip(tipper_id: u64, tipee_id: u64, is_anti: bool) -> TipResult {
    if tipper_id == tipee_id {
        return TipResult::SameId;
    }

    let tipper_user = get_user(tipper_id);
    // make sure the reciever exists
    let _tipee_user = get_user(tipee_id);
    if !is_anti && tipper_user.tips == 0 {
        return TipResult::NoTips;
    } else if is_anti && tipper_user.anti_tips == 0 {
        return TipResult::NoTips;
    }

    update_user(tipper_id, TipAction::Give(is_anti));
    update_user(tipee_id, TipAction::Recieve(is_anti));

    TipResult::Ok
}

fn get_data() -> Data {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let path = Path::new("./data/tips/tips.json");
    let json = match OpenOptions::new().write(true).read(true).open(path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(path).expect("Error creating file"),
    };

    match serde_json::from_reader(json) {
        Ok(j) => j,
        Err(_) => Data {
            reset_time: time.as_secs() + SECONDS_IN_WEEK,
            users: Vec::new(),
        }
    }
}

fn get_log() -> Tips {
    let log_path = Path::new("./data/tips/log.json");
    let json_log = match OpenOptions::new().write(true).read(true).open(log_path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(log_path).expect("Error creating log file"),
    };

    match serde_json::from_reader(json_log) {
        Ok(j) => j,
        Err(_) => Tips {
            tips: Vec::new()
        }
    }
}

fn write_data(tip_data: &Data) -> TipResult {
    let path = Path::new("./data/tips/tips.json");
    let writer = BufWriter::new(OpenOptions::new().write(true).open(path).unwrap());
    match serde_json::to_writer(writer, tip_data) { 
        Ok(_) => TipResult::Ok,
        Err(_) => TipResult::WriteErr,
    }
}

fn write_log(log_data: &Tips) -> TipResult {
    let log_path = Path::new("./data/tips/log.json");
    let log_writer = BufWriter::new(OpenOptions::new().write(true).open(log_path).unwrap());
    match serde_json::to_writer(log_writer, log_data) {
        Ok(_) => TipResult::Ok,
        Err(_) => TipResult::WriteErr,
    }
}

fn check_tips() -> TipResult {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let data = get_data();
    if time.as_secs() > data.reset_time {
        return reset_tips(time, data);
    }
    TipResult::Ok
}

fn reset_tips(time: std::time::Duration, old_data: Data) -> TipResult {
    let old_time = old_data.reset_time;
    let new_users = old_data.users.into_iter().map(|mut user| {
                                                   user.tips = WEEKLY_TIPS;
                                                   user.anti_tips = WEEKLY_ANTI_TIPS;
                                                   user.week_gross = 0;
                                                   user.week_net = 0;
                                                   user
                                                }).collect();
    let mut new_time = old_time;
    while time.as_secs() > old_time {
        new_time += SECONDS_IN_WEEK
    };

    write_data(&Data {
        reset_time: new_time,
        users: new_users,
    })
}

fn create_user(id: u64, mut data: Data) -> User {
    let mut user = User::default();
    user.user_id = id;
    
    data.users.push(user);
    write_data(&data);

    get_user(id)
}

fn get_user(id: u64) -> User {
    let data = get_data();
    let exists = data.users.iter().any(|x| x.user_id == id);

    if exists {
        data.users.iter().find(|x| x.user_id == id).unwrap().clone()
    } else {
        create_user(id, data)
    }

}

fn update_user(id: u64, action: TipAction) -> TipResult {
    let mut data = get_data();
    {
        let user = match data.users.iter_mut().find(|user| user.user_id == id) {
            Some(u) => u,
            None => panic!("Something terrible has happened"),
        };

        match action {
            TipAction::Give(is_anti) => {
                if is_anti {
                    user.anti_tips -= 1;
                    user.anti_tips_given += 1;
                } else {
                    user.tips -= 1;
                    user.tips_given += 1;
                }
            }
            TipAction::Recieve(is_anti) => {
                if is_anti {
                    user.week_net -= 1;
                    user.lifetime_net -= 1;
                } else {
                    user.week_gross += 1;
                    user.week_net += 1;
                    user.lifetime_net += 1;
                    user.lifetime_gross += 1;
                }
            }
        }
    }
    write_data(&data)
}
