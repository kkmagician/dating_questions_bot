use crate::bot::room::*;
use crate::bot::constants::*;
use crate::telegram::helpers::*;
use crate::telegram::messages::send_message;

use serde::{Deserialize, Serialize};
use reqwest::Client;
use redis::Commands;

pub struct TgMethods;
impl TgMethods {
    pub const GET_UPDATES: &'static str = "getUpdates";
    pub const SEND_MESSAGE: &'static str = "sendMessage";
    pub const EDIT_MESSAGE_REPLY_MARKUP: &'static str = "editMessageReplyMarkup";
    pub const ANSWER_CALLBACK_QUERY: &'static str = "answerCallbackQuery";
}

#[derive(Deserialize, Debug)]
pub struct TgResult {
    pub result: Vec<TgUpdate>
}

#[derive(Deserialize, Debug)]
pub struct TgError {
    pub description: String
}

#[derive(Deserialize, Debug)]
pub struct InlineQuery {
    query: String
}

#[derive(Deserialize, Debug)]
pub struct CallbackQuery {
    id: String,
    message: Option<TgMessage>,
    data: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct TgUser {
    pub(crate) id: i32,
    first_name: String,
    last_name: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct TgChat {
    id: i64
}

#[derive(Serialize, Debug)]
pub struct ReplyKeyboardMarkup {
    pub(crate) keyboard: Vec<Vec<String>>,
    pub(crate) one_time_keyboard: bool
}

#[derive(Serialize, Debug)]
pub struct OutgoingKeyboardMessage {
    pub(crate) chat_id: i32,
    pub(crate) text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reply_markup: Option<ReplyKeyboardMarkup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parse_mode: Option<String>
}

impl OutgoingKeyboardMessage {
    pub(crate) fn with_text(chat_id: i32, text: &str) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage {
            chat_id,
            text: String::from(text),
            reply_markup: None,
            parse_mode: None
        }
    }

    pub(crate) fn with_keyboard(chat_id: i32, text: &str, keyboard: Vec<Vec<String>>) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage {
            chat_id,
            text: String::from(text),
            reply_markup: Some(ReplyKeyboardMarkup { keyboard, one_time_keyboard: true }),
            parse_mode: None
        }
    }

    pub(crate) fn welcome_message(chat_id: i32) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_keyboard(
        chat_id,
        Messages::WELCOME,
        Keys::welcome()
        )
    }

    pub(crate) fn join_room(chat_id: i32) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_text(
            chat_id,
            Messages::INSERT_ROOM_ID
        )
    }

    pub(crate) fn create_select_pack(chat_id: i32, packs: Vec<String>) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_keyboard(
            chat_id,
            Messages::CHOOSE_PACK,
            packs.iter().map(|pack| vec![pack.to_string()]).collect()
        )
    }

    pub(crate) fn wrong_room_id(chat_id: i32) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_text(chat_id, Messages::WRONG_ROOM_ID)
    }

    pub(crate) fn error(chat_id: i32) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_text(chat_id, Messages::ERROR)
    }

    pub(crate) fn room_id_message(chat_id: i32, room_id: &String) -> OutgoingKeyboardMessage {
        OutgoingKeyboardMessage::with_text(
            chat_id,
            format!("{}\nID комнаты: {}", Messages::WAITING_FOR_PARTNER, room_id).as_str()
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct SentMessageResponse {
    pub ok: bool,
    pub result: Option<SentMessage>,
    pub description: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct SentMessage {
    pub message_id: i32
}

#[derive(Serialize, Debug)]
pub struct CallbackQueryAnswer {
    callback_query_id: String,
    text: Option<String>
}

async fn answer_callback_query(bot_token: &str, client: &Client, callback_query_id: String, idx: u8)
                               -> Result<(), Box<dyn std::error::Error>> {
    let url = create_tg_url(bot_token, TgMethods::ANSWER_CALLBACK_QUERY);
    let answer = CallbackQueryAnswer {
        callback_query_id,
        text: EMOJIS.get(idx as usize).map(|x: &&str| format!("Оценка: {}", x))
    };

    client.post(&url)
        .json(&answer)
        .send()
        .await?;

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct OutgoingInlineKeyboardMessage {
    chat_id: i32,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_markup: Option<InlineKeyboardMarkup>
}

impl OutgoingInlineKeyboardMessage {
    fn create_eval_keys(typ: u8, selected_key: Option<u8>)
        -> Vec<InlineKeyboardButton> {
        let selected_idx = selected_key.unwrap_or(99);
        EMOJIS.iter()
            .enumerate()
            .map(|(i, &x)| InlineKeyboardButton {
                text: if i == selected_idx as usize {
                    format!("({})", x)
                } else {
                    format!("{}", x)
                },
                callback_data: serde_json::to_string(&CallbackData {
                    idx: i as u8,
                    typ
                }).unwrap()
            })
            .collect()
    }

    pub(crate) fn with_eval_keys(chat_id: i32, text: &str, typ: u8)
        -> OutgoingInlineKeyboardMessage {
        let keys = OutgoingInlineKeyboardMessage::create_eval_keys(typ, None);

        OutgoingInlineKeyboardMessage {
            chat_id,
            text: text.parse().unwrap(),
            reply_markup: Some(InlineKeyboardMarkup{ inline_keyboard: vec![keys] })
        }
    }
}

#[derive(Serialize, Debug)]
pub struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>
}

#[derive(Deserialize, Debug)]
pub struct TgMessage {
    message_id: i32,
    pub(crate) from: TgUser,
    chat: TgChat,
    date: i32,
    pub(crate) text: Option<String>,
    entities: Option<Vec<MessageEntity>>
}

#[derive(Deserialize, Debug)]
pub struct MessageEntity {
    #[serde(rename = "type")]
    typ: String
}

#[derive(Serialize, Debug)]
pub struct InlineKeyboardButton {
    text: String,
    callback_data: String
}

#[derive(Serialize, Debug)]
pub struct EditedReplyInlineMarkup {
    chat_id: i64,
    message_id: i32,
    reply_markup: Option<InlineKeyboardMarkup>
}

impl EditedReplyInlineMarkup {
    async fn edit(&self, bot_token: &str, client: &Client) -> Result<(), reqwest::Error> {
        let url = create_tg_url(bot_token, TgMethods::EDIT_MESSAGE_REPLY_MARKUP);
        let response = client.post(&url).json(self).send().await;

        if response.is_err() {
            log::error!("{:?}", response);
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CallbackData {
    idx: u8,
    typ: u8
}

impl CallbackData {
    fn match_type(&self) -> CallbackMessageType {
        match self.typ {
            1 => CallbackMessageType::Importance,
            2 => CallbackMessageType::Evaluation,
            _ => CallbackMessageType::Error
        }
    }

    fn role_has_all_callback_keys(
        role: &String,
        room_id: &String,
        redis: &mut redis::Connection
    ) -> Result<bool, redis::RedisError> {
        let set_values: Vec<i32> = redis.hget(
            format!("room:{}", room_id),
            &[
                format!("{}_importance", role),
                format!("{}_evaluation", role),
            ]
        )?;

        Ok(set_values.len() == 2)
    }

    fn set_value_for_role(
        role: &String,
        message_type: &CallbackMessageType,
        value: u8,
        room_id: &String,
        redis: &mut redis::Connection)
        -> Result<bool, redis::RedisError> {
        if (role == Role::CREATOR || role == Role::VISITOR) && value < 5 {
            let redis_field = format!("{}_{}", role, format!("{:?}", message_type).to_lowercase()); // creator_importance, ...
            let previous_has_all_keys = CallbackData::role_has_all_callback_keys(role, room_id, redis)?;
            redis.hset(format!("room:{}", room_id), redis_field, value)?;
            let new_has_all_keys = CallbackData::role_has_all_callback_keys(role, room_id, redis)?;

            if !previous_has_all_keys && new_has_all_keys {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub(crate) async fn handle_callback(&self, id: &String, user_id: i64, message_id: i32, redis: &mut redis::Connection, client: &Client, bot_token: &str)
        -> Result<(), Box<dyn std::error::Error>> {
        let context = Context::get(user_id as i32, redis)?;

        if context == Context::IN_ROOM || context == Context::WAITING_FOR_ANSWER {
            let user_room = UserRoom::get(user_id as i32, redis)?;

            let message_type = self.match_type();
            let send_next_question_keys =
                CallbackData::set_value_for_role(&user_room.role, &message_type, self.idx, &user_room.id, redis)?;

            let inline_keyboard = vec![OutgoingInlineKeyboardMessage::create_eval_keys(self.typ, Some(self.idx))];
            let edited_keys = EditedReplyInlineMarkup {
                chat_id: user_id,
                message_id,
                reply_markup: Some(InlineKeyboardMarkup { inline_keyboard })
            };

            edited_keys.edit(bot_token, client).await?;

            if send_next_question_keys {
                Context::set_context(user_id as i32, Context::WAITING_FOR_ANSWER, redis)?;

                let is_ready_for_next_msg = OutgoingKeyboardMessage {
                    chat_id: user_id as i32,
                    text: String::from(Messages::READY_FOR_NEXT),
                    reply_markup: Some(ReplyKeyboardMarkup {
                        keyboard: vec![vec![Keys::READY.to_string()]],
                        one_time_keyboard: false
                    }),
                    parse_mode: None
                };

                send_message(
                    &create_tg_url(bot_token, TgMethods::SEND_MESSAGE),
                    &is_ready_for_next_msg,
                    client
                ).await?;
            }
        }

        answer_callback_query(bot_token, client, id.to_string(), self.idx).await?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct TgUpdate {
    pub(crate) update_id: i32,
    pub(crate) message: Option<TgMessage>,
    edited_message: Option<TgMessage>,
    inline_query: Option<InlineQuery>,
    callback_query: Option<CallbackQuery>
}

impl TgUpdate {
    pub(crate) fn handle_message_type(&self, redis: &mut redis::Connection) -> UpdateType {
        let user_id = self.message.as_ref().map(|x| x.from.id);
        let message_text = self.message.as_ref().and_then(|x| x.text.as_ref());

        if self.callback_query.is_some() {
            let query = self.callback_query.as_ref().unwrap();
            let callback_query_id = &query.id;
            let data = query.data.as_ref().and_then(|x| serde_json::from_str::<CallbackData>(&x).ok());
            let message_id = query.message.as_ref().map(|x| x.message_id);
            let chat_id = query.message.as_ref().map(|x| x.chat.id);

            log::info!("{:?}, {:?}, {:?}", chat_id, message_id, data);

            match (chat_id, message_id, data) {
                (Some(uid), Some(cid), Some(d)) =>
                    UpdateType::Callback(uid, cid, d, callback_query_id.to_string()),
                _ => UpdateType::Error
            }
        } else if self.message.as_ref()
            .and_then(|x| x.entities.as_ref())
            .map(|x|
                x.iter().any(|y| y.typ == "bot_command")
            ) == Some(true) {
            if user_id.is_some() {
                Context::reset(user_id.unwrap(), redis);
                let _: Result<(), _> = redis.del(format!("user:{}:room", user_id.unwrap()));
            }
            UpdateType::Start
        } else if message_text == Some(&Keys::WELCOME[0].to_string()) {
            UpdateType::JoinExisting
        } else if message_text == Some(&Keys::WELCOME[1].to_string()) {
            UpdateType::Create
        } else {
            if user_id.is_some() {
                let context = Context::get(user_id.unwrap(), redis);
                if context.is_ok() {
                    let context_str = context.unwrap();
                    if context_str == Context::SELECT_PACK {
                        UpdateType::NewRoom
                    } else if context_str == Context::INSERT_ID {
                        UpdateType::InsertId
                    } else if context_str == Context::WAITING_FOR_ANSWER && message_text == Some(&Keys::READY.to_string()) {
                        UpdateType::WaitingForOther
                    } else if context_str == Context::WAITING_FOR_RESULTS {
                        UpdateType::WaitingForResults
                    } else {
                        UpdateType::Error
                    }
                } else {
                    UpdateType::Error
                }
            }
            else {
                UpdateType::Other
            }
        }
    }
}