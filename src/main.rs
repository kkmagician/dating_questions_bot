mod bot;
mod tools;
mod telegram;

use crate::tools::read_key_env;
use crate::bot::constants::*;
use crate::bot::room::*;
use crate::telegram::structures::*;
use crate::telegram::helpers::*;
use crate::telegram::messages::*;

use std::borrow::Borrow;
use reqwest::Client;
use redis::Commands;
use simple_logger::SimpleLogger;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let token = read_key_env("TG_TOKEN").expect("No TG_TOKEN found!");
    let ch_url = read_key_env("CH_URL").expect("No CH_URL found!");
    let client = reqwest::Client::new();
    let mut redis = redis::Client::open(read_key_env("REDIS")
        .unwrap())?
        .get_connection()?;

    log::info!("Started the bot");

    longpoll(&token, &client, &mut redis, &ch_url).await?;

    Ok(())
}

async fn get_updates(bot_token: &str, client: &Client, latest_update_id: i32)
    -> Result<TgResult, Box<dyn std::error::Error>> {
    let url = create_tg_url(bot_token, TgMethods::GET_UPDATES);
    let offset: String = format!("{}", latest_update_id + 1);
    let params: [(&str, &str); 2] = [
        ("offset", &offset),
        ("timeout", "60")
    ];

    let res = client.get(&url)
        .query(&params)
        .send().await?
        .bytes().await?;

    let tg_response = serde_json::from_slice::<TgResult>(&res);

    if tg_response.is_err() {
        match serde_json::from_slice::<TgError>(&res) {
            Ok(tg_error) => log::error!("{:?}", tg_error),
            Err(err)       => log::error!("{:?}", err)
        }
    }

    Ok(tg_response?)
}

async fn longpoll(bot_token: &str, client: &Client, redis: &mut redis::Connection, ch_url: &String)
    -> Result<(), Box<dyn std::error::Error>> {
    let mut latest_update_id: i32 = 0;
    let url = create_tg_url(bot_token, TgMethods::SEND_MESSAGE);

    loop {
        let updates = get_updates(bot_token, client, latest_update_id).await?;

        for update in updates.result {
            let message_type = update.handle_message_type(redis);

            let message = update.message.borrow();

            if message.is_some() {
                log::info!("{:?}", message.as_ref().unwrap());
            }

            let chat_id = message.as_ref().map(|x| x.from.id);

            latest_update_id = update.update_id;
            redis.set(RedisKeys::LATEST_MESSAGE, latest_update_id)?;

            match &message_type {
                UpdateType::Callback(chat, message, d, id) => {
                    d.handle_callback(id, *chat, *message, redis, client, bot_token).await?;
                },
                _ => ()
            }

            if chat_id.is_some() {
                let user_id = chat_id.unwrap();

                let response: Option<OutgoingKeyboardMessage> = match message_type {
                    UpdateType::Start => Some(OutgoingKeyboardMessage::welcome_message(user_id)),
                    UpdateType::JoinExisting => {
                        let msg = OutgoingKeyboardMessage::join_room(user_id);
                        Context::set_context(user_id, Context::INSERT_ID, redis)?;

                        Some(msg)
                    },
                    UpdateType::Create => {
                        let packs: Vec<String> = redis.smembers(RedisKeys::PACKS)?;
                        let msg = OutgoingKeyboardMessage::create_select_pack(user_id, packs);
                        Context::set_context(user_id, Context::SELECT_PACK, redis)?;

                        Some(msg)
                    }
                    UpdateType::NewRoom => {
                        let pack_opt = message.as_ref().and_then(|x| x.text.as_ref());

                        if pack_opt.is_some() {
                            let pack = pack_opt.unwrap();
                            let is_existing_pack: bool = redis.sismember(RedisKeys::PACKS, pack)?;
                            if is_existing_pack {
                                let room_id = Room::create(user_id, &pack, redis);
                                let msg = OutgoingKeyboardMessage::room_id_message(user_id, &room_id);
                                Context::set_context(user_id, Context::WAITING_FOR_PARTNER, redis)?;

                                Some(msg)
                            } else {
                                Some(OutgoingKeyboardMessage::with_text(chat_id.unwrap(), Messages::ERROR))
                            }
                        } else {
                            Some(OutgoingKeyboardMessage::with_text(chat_id.unwrap(), Messages::ERROR))
                        }
                    }
                    UpdateType::InsertId => {
                        let id_opt = message.as_ref().and_then(|x| x.text.as_ref());

                        if id_opt.is_some() {
                            let id = id_opt.unwrap();
                            let room_id = format!("room:{}", id);
                            let is_existing_room: bool = redis.exists(&room_id)?;
                            let is_vacant_room: bool = !redis.hexists(&room_id, "visitor_id")?;

                            if is_existing_room && is_vacant_room {
                                log::info!("Entering room {}, user {}", &room_id, user_id);

                                Room::enter(&id, user_id, redis)?;
                                Room::start(&id, redis, client, &url, ch_url).await?;
                                None
                            } else {
                                Some(OutgoingKeyboardMessage::wrong_room_id(user_id))
                            }
                        } else {
                            Some(OutgoingKeyboardMessage::wrong_room_id(user_id))
                        }
                    },
                    UpdateType::WaitingForOther => {
                        let user_room = UserRoom::get(user_id, redis)?;
                        if user_room.set_ready_time(redis)? {
                            Room::write_data(&user_room.id, redis, client, ch_url).await?;

                            let idx = Room::prepare_for_next_question(&user_room.id, redis)?;
                            let users: Vec<i32> = redis.hget(format!("room:{}", &user_room.id), &["creator_id", "visitor_id"])?;
                            let pack: String = redis.hget(format!("room:{}", &user_room.id), "pack")?;
                            send_question_messages([users[0], users[1]], &pack, idx, redis, client, &url, &user_room.id, ch_url).await?;

                            None
                        } else {
                            Some(OutgoingKeyboardMessage::with_text(user_id, Messages::WAITING_FOR_PARTNER_EVAL))
                        }
                    },
                    UpdateType::WaitingForResults => {
                      Some(OutgoingKeyboardMessage::with_text(user_id, Messages::WAIT_A_MOMENT))
                    },
                    UpdateType::Error => {
                        Some(OutgoingKeyboardMessage::error(user_id))
                    },
                    _ => Some(OutgoingKeyboardMessage::error(user_id))
                };

                if response.is_some() {
                    send_message(&url, &response.unwrap(), client).await?;
                }
            }
        }

        log::info!("Latest update: {}", latest_update_id);
    }

    Ok(())
}