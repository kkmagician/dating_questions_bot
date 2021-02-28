use crate::telegram::structures::*;
use crate::bot::room::*;
use crate::bot::constants::*;
use crate::bot::report::ReportData;

use reqwest::Client;
use redis::Commands;
use serde::Serialize;

pub(crate) async fn send_question_messages(
    user_ids: [i32; 2],
    pack: &String,
    idx: u16,
    redis: &mut redis::Connection,
    client: &Client,
    url: &str,
    room_id: &String,
    ch_url: &String
) -> Result<(), Box<dyn std::error::Error>> {
    let pack_message: Option<String> = redis.lindex(format!("pack:{}", pack), idx as isize)?;
    let pack_len: u16 = redis.llen(format!("pack:{}", pack))?;

    if pack_message.is_some() {
        let message_txt = pack_message.unwrap();
        let header = format!("<b>📒Вопрос {} из {}:</b>\n", idx + 1, pack_len);

        for &user_id in user_ids.iter() {
            Context::set_context(user_id, Context::IN_ROOM, redis)?;

            let message = OutgoingKeyboardMessage {
                chat_id: user_id,
                text: format!("{}{}", &header, &message_txt),
                reply_markup: None,
                parse_mode: Some("HTML".to_string())
            };
            let importance = OutgoingInlineKeyboardMessage::with_eval_keys(user_id, Messages::ANSWER_IMPORTANCE, 1);
            let evaluation = OutgoingInlineKeyboardMessage::with_eval_keys(user_id, Messages::ANSWER_EVALUATION, 2);

            send_message(url, &message, client).await?;
            send_message(url, &importance, client).await?;
            send_message(url, &evaluation, client).await?;
        }
    } else {
        for &user_id in user_ids.iter() {
            let final_message = OutgoingKeyboardMessage {
                chat_id: user_id,
                text: String::from(Messages::EVALUATING_RESULTS),
                reply_markup: None,
                parse_mode: None
            };

            Context::set_context(user_id, Context::WAITING_FOR_RESULTS, redis)?;
            send_message(url, &final_message, client).await?;
        }

        let report = ReportData::get(room_id, client, ch_url).await?;

        for &user_id in user_ids.iter() {
            let user_role = Role::get(user_id, redis)?;
            let report_string = report.generate_report(&user_role);
            let message = OutgoingKeyboardMessage {
                chat_id: user_id,
                text: report_string,
                reply_markup: Some(ReplyKeyboardMarkup {
                    keyboard: Keys::welcome(),
                    one_time_keyboard: true
                }),
                parse_mode: Some("HTML".to_string())
            };

            send_message(url, &message, client).await?;
            Context::reset(user_id, redis)?;
            let _: Result<(), redis::RedisError> = redis.del(format!("user:{}:room", user_id));
        }

        Room::clear(room_id, redis)?
    }

    Ok(())
}

pub async fn send_message<T: Serialize + ?Sized>(url: &str, message: &T, client: &Client)
    -> Result<i32, reqwest::Error> {
    let res = client.post(url)
        .json(message)
        .send()
        .await?
        .json::<SentMessageResponse>()
        .await;

    if res.is_err() {
        log::error!("{:?}", res);
    }

    res.map( |x| {
        match x {
            SentMessageResponse{
                ok: true,
                result: Some(SentMessage { message_id: id }),
                ..
            } => id,
            _ => 0
        }
    } )
}
