mod bot;
mod tools;
mod telegram;

use crate::tools::read_key_env;
use crate::bot::constants::*;
use crate::bot::handlers::Handlers;
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

    longpoll(&token, &client, &mut redis, &ch_url).await
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

async fn handle_updates(
    update: &TgUpdate,
    bot_token: &str,
    client: &Client,
    redis: &mut redis::Connection,
    ch_url: &String,
    url: &String
) -> Result<(), Box<dyn std::error::Error>> {
    let message_type = update.handle_message_type(redis)?;
    let message = update.message.borrow();

    if message.is_some() {
        log::info!("{:?}", message.as_ref().unwrap());
    }

    let chat_id = message.as_ref().map(|x| x.from.id);

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
            UpdateType::JoinExisting => Handlers::join_existing(user_id, redis)?,
            UpdateType::Create => Handlers::create(user_id, redis)?,
            UpdateType::NewRoom => Handlers::new_room(user_id, message, redis)?,
            UpdateType::InsertId =>
                Handlers::insert_id(user_id, message, client, redis, url, ch_url).await?,
            UpdateType::WaitingForOther =>
                Handlers::waiting_for_answer(user_id, client, redis, url, ch_url).await?,
            UpdateType::WaitingForResults =>
                Some(OutgoingKeyboardMessage::with_text(user_id, Messages::WAIT_A_MOMENT)),
            UpdateType::Help =>
                Some(OutgoingKeyboardMessage::with_text(user_id, Messages::HELP)),
            UpdateType::UnknownCommand =>
                Some(OutgoingKeyboardMessage::with_text(user_id, Messages::ERROR_UNKNOWN_COMMAND)),
            UpdateType::Error => Some(OutgoingKeyboardMessage::error(user_id)),
            _ => Some(OutgoingKeyboardMessage::error(user_id))
        };

        if response.is_some() {
            send_message(url, &response.unwrap(), client).await?;
        }
    }

    Ok(())
}

async fn longpoll(bot_token: &str, client: &Client, redis: &mut redis::Connection, ch_url: &String)
    -> Result<(), Box<dyn std::error::Error>> {
    let mut latest_update_id: i32 = 0;
    let url = create_tg_url(bot_token, TgMethods::SEND_MESSAGE);

    loop {
        let updates = get_updates(bot_token, client, latest_update_id).await?;

        for update in updates.result {
            let upd = handle_updates(&update, bot_token, client, redis, ch_url, &url).await;

            if upd.is_err() {
                let user_id = &update.message.map(|m| m.from.id);

                if user_id.is_some() {
                    send_message(&url, &OutgoingKeyboardMessage::internal_error(user_id.unwrap()), client).await?;
                }

                log::error!("{:?}", upd);
            }

            latest_update_id = update.update_id;
            redis.set(RedisKeys::LATEST_MESSAGE, latest_update_id)?;
            log::info!("Latest update: {}", latest_update_id);
        }
    }

    Ok(())
}
