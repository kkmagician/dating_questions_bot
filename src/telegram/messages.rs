use crate::telegram::structures::*;
use crate::bot::room::*;
use crate::bot::constants::*;
use crate::bot::report::ReportData;

use reqwest::Client;
use redis::Commands;
use serde::Serialize;

#[derive(Debug)]
pub(crate) struct QuestionMessage {
    header: String,
    message: String
}

impl QuestionMessage {
    pub fn create_text(&self) -> String {
        format!("{}{}", self.header, self.message)
    }

    fn get(pack: &String, idx: u16, redis: &mut redis::Connection)
           -> Result<Option<QuestionMessage>, redis::RedisError> {
        let pack_message: Option<String> = redis.lindex(format!("pack:{}", pack), idx as isize)?;

        if pack_message.is_some() {
            let pack_len: u16 = redis.llen(format!("pack:{}", pack))?;
            let header = format!("<b>📒Вопрос {} из {}:</b>\n", idx + 1, pack_len);
            let message = pack_message.unwrap();

            Ok(Some(QuestionMessage { header, message }))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_room_id(room_id: &String, redis: &mut redis::Connection)
        -> Result<Option<QuestionMessage>, redis::RedisError> {
        let pack: Option<String> = redis.hget(Room::key(room_id), "pack")?;
        let idx: Option<u16> = redis.hget(Room::key(room_id), "idx")?;

        match (pack, idx) {
            (Some(pack), Some(idx)) => QuestionMessage::get(&pack, idx, redis),
            _ => Ok(None)
        }
    }

    pub async fn send(
        &self,
        user_id: i32,
        room_id: &String,
        redis: &mut redis::Connection,
        client: &Client,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Context::set_context(user_id, Context::IN_ROOM, redis)?;

        let message = OutgoingKeyboardMessage {
            chat_id: user_id,
            text: self.create_text(),
            reply_markup: None,
            parse_mode: Some("HTML".to_string())
        };
        let importance = OutgoingInlineKeyboardMessage::with_eval_keys(user_id, Messages::ANSWER_IMPORTANCE, 1, room_id);
        let evaluation = OutgoingInlineKeyboardMessage::with_eval_keys(user_id, Messages::ANSWER_EVALUATION, 2, room_id);

        send_message(url, &message, client).await?;
        send_message(url, &importance, client).await?;
        send_message(url, &evaluation, client).await?;

        Ok(())
    }
}

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
    let question_message = QuestionMessage::get(pack, idx, redis)?;

    if question_message.is_some() {
        let question_message = question_message.unwrap();

        for &user_id in user_ids.iter() {
            &question_message.send(user_id, room_id, redis, client, url).await?;
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
            SentMessageResponse {
                ok: true,
                result: Some(SentMessage { message_id: id }),
                ..
            } => id,
            _ => 0
        }
    } )
}
