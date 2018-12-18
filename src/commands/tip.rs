use serenity::utils::MessageBuilder;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_WEEK: u64 = 604800;
const WEEKLY_TIPS: u8 = 7;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    user_id: u64,
    lifetime_tips: u32,
    week_tips: u8,
    tips_to_give: u8,
    tips_given: u32,
}

#[derive(Serialize, Deserialize)]
struct Data {
    reset_time: u64,
    users: Vec<User>,
}

command!(tip(_ctx, msg, msg_args) {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let path = Path::new("./data/tips/tips.json");
    let json = match OpenOptions::new().write(true).read(true).open(path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(path).expect("Error creating file"),
    };

    let mut data: Data = match serde_json::from_reader(json) {
        Ok(j) => j,
        Err(_) => Data {
            reset_time: time.as_secs() + SECONDS_IN_WEEK,
            users: Vec::new(),
        }
    };

    if time.as_secs() > data.reset_time {
        data.users.iter_mut().for_each(|x| {x.tips_to_give = WEEKLY_TIPS; x.week_tips = 0;});
        while data.reset_time < time.as_secs() {
            data.reset_time += SECONDS_IN_WEEK;
        }
    }

    let tipper_id = msg.author.id;
    let tipper = *tipper_id.as_u64();
    let mut tipee_id: serenity::model::id::UserId;
    let is_tip = match msg_args.single_n::<serenity::model::id::UserId>() {
        Ok(_) => true,
        Err(_) => false,
    };
    if !is_tip {
        let exists = data.users.iter().any(|x| x.user_id == tipper);
        let mut tipper_user = if exists {
            data.users.iter_mut().find(|x| x.user_id == tipper).unwrap()
        } else {
            data.users.push(User {
                user_id: tipper,
                lifetime_tips: 0,
                week_tips: 0,
                tips_to_give: WEEKLY_TIPS,
                tips_given: 0,
            });
            let len = data.users.len();
            data.users.get_mut(len - 1).unwrap()
        };

        let mut content = MessageBuilder::new()
            .push("```md\n")
            .push(format!("### You have:\n* Lifetime recieved tips: {}", tipper_user.lifetime_tips))
            .push(format!("\n* Tips recieved this week: {}", tipper_user.week_tips))
            .push(format!("\n* Lifetime tips given: {}", tipper_user.tips_given))
            .push(format!("\n* Tips to give this week: {}", tipper_user.tips_to_give))
            .push(format!("\n### Usage ###\n -tip @SomeWellDeservingPersonHere\n```"))
            .build();
        let _ = msg.reply(&content);
    } else {
        tipee_id = match msg_args.single::<serenity::model::id::UserId>() {
            Ok(u) => u,
            Err(_) => panic!(),
        };

        let tipee = *tipee_id.as_u64();

        if tipper == tipee {
            let _ = msg.reply("You can't tip yourself lol");
        } else {
            let mut _tipper_to_give = 0;
            let mut _tipper_given = 0;
            let mut _tipee_tips = (0, 0);
            {
                let exists = data.users.iter().any(|x| x.user_id == tipper);
                let mut tipper_user = if exists {
                    data.users.iter_mut().find(|x| x.user_id == tipper).unwrap()
                } else {
                    data.users.push(User {
                        user_id: tipper,
                        lifetime_tips: 0,
                        week_tips: 0,
                        tips_to_give: WEEKLY_TIPS,
                        tips_given: 0,
                    });
                    let len = data.users.len();
                    data.users.get_mut(len - 1).unwrap()
                };



                if tipper_user.tips_to_give == 0 {
                    let _ = msg.reply("You are out of tips this week, try again next week");
                }

                tipper_user.tips_to_give -= 1;
                tipper_user.tips_given += 1;
                _tipper_to_give = tipper_user.tips_to_give;
                _tipper_given = tipper_user.tips_given;
            }
            {

                let exists = data.users.iter().any(|x| x.user_id == tipee);
                let mut tipee_user = if exists {
                    data.users.iter_mut().find(|x| x.user_id == tipee).unwrap()
                } else {
                    data.users.push(User {
                        user_id: tipee,
                        lifetime_tips: 0,
                        week_tips: 0,
                        tips_to_give: WEEKLY_TIPS,
                        tips_given: 0,
                    });
                    let len = data.users.len();
                    data.users.get_mut(len - 1).unwrap()
                };

                tipee_user.week_tips += 1;
                tipee_user.lifetime_tips += 1;
                _tipee_tips = (tipee_user.week_tips, tipee_user.lifetime_tips);
            }

            let mut content = MessageBuilder::new()
                .mention(&tipper_id)
                .push(" Tipped ")
                .mention(&tipee_id)
                .push("\n")
                .mention(&tipper_id)
                .push(format!(" has given {} lifetime tips and has {} remaining this week \n",
                              _tipper_given,
                              _tipper_to_give))
                .mention(&tipee_id)
                .push(format!(" has recieved {} tips this week and {} lifetime tips", _tipee_tips.0, _tipee_tips.1))
                .build();

            println!("{:?} -> {:?}" , tipper, tipee);
            let _ = msg.channel_id.say(&content);
            let writer = BufWriter::new(OpenOptions::new().write(true).open(path).unwrap());
            let _  = serde_json::to_writer(writer, &data).unwrap();
        }
    }
});
