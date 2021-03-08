use crate::bot::constants::*;
use crate::bot::room::*;
use crate::telegram::structures::*;
use crate::telegram::messages::send_question_messages;

use reqwest::Client;
use redis::Commands;

pub struct Handlers;
impl Handlers {
    pub(crate) fn join_existing(user_id: i32, redis: &mut redis::Connection)
        -> Result<Option<OutgoingKeyboardMessage>, redis::RedisError> {
        let msg = OutgoingKeyboardMessage::join_room(user_id);
        Context::set_context(user_id, Context::INSERT_ID, redis)?;

        Ok(Some(msg))
    }

    pub(crate) fn create(user_id: i32, redis: &mut redis::Connection)
        -> Result<Option<OutgoingKeyboardMessage>, redis::RedisError> {
        let packs: Vec<String> = redis.smembers(RedisKeys::PACKS)?;
        let msg = OutgoingKeyboardMessage::create_select_pack(user_id, packs);
        Context::set_context(user_id, Context::SELECT_PACK, redis)?;

        Ok(Some(msg))
    }

    pub(crate) fn new_room(
        user_id: i32,
        message: &Option<TgMessage>,
        redis: &mut redis::Connection
    ) -> Result<Option<OutgoingKeyboardMessage>, redis::RedisError> {
        let pack_opt = message.as_ref().and_then(|x| x.text.as_ref());

        if pack_opt.is_some() {
            let pack = pack_opt.unwrap();
            let is_existing_pack: bool = redis.sismember(RedisKeys::PACKS, pack)?;
            if is_existing_pack {
                let room_id = Room::create(user_id, &pack, redis);
                let msg = OutgoingKeyboardMessage::room_id_message(user_id, &room_id);
                Context::set_context(user_id, Context::WAITING_FOR_PARTNER, redis)?;

                Ok(Some(msg))
            } else {
                Ok(Some(OutgoingKeyboardMessage::with_text(user_id, Messages::ERROR_PACK_DOES_NOT_EXIST)))
            }
        } else {
            Ok(Some(OutgoingKeyboardMessage::with_text(user_id, Messages::ERROR)))
        }
    }

    pub(crate) async fn insert_id (
        user_id: i32,
        message: &Option<TgMessage>,
        client: &Client,
        redis: &mut redis::Connection,
        url: &str,
        ch_url: &String
    ) -> Result<Option<OutgoingKeyboardMessage>, Box<dyn std::error::Error>> {
        let id_opt = message.as_ref().and_then(|x| x.text.as_ref());

        if id_opt.is_some() {
            let id = id_opt.unwrap();
            let room_id = Room::key(id);
            let is_existing_room: bool = redis.exists(&room_id)?;
            let visitor_id: Option<i32> = redis.hget(&room_id, "visitor_id")?;
            let is_vacant_room = visitor_id.is_none() || visitor_id.map_or(false, |id| id == user_id);

            if is_existing_room && is_vacant_room {
                log::info!("Entering room {}, user {}", &room_id, user_id);

                Room::enter(&id, user_id, redis)?;
                Room::start(&id, redis, client, url, ch_url).await?;
                Ok(None)
            } else {
                Ok(Some(OutgoingKeyboardMessage::wrong_room_id(user_id)))
            }
        } else {
            Ok(Some(OutgoingKeyboardMessage::wrong_room_id(user_id)))
        }
    }

    pub(crate) async fn waiting_for_answer(
        user_id: i32,
        client: &Client,
        redis: &mut redis::Connection,
        url: &String,
        ch_url: &String
    ) -> Result<Option<OutgoingKeyboardMessage>, Box<dyn std::error::Error>> {
        let user_room = UserRoom::get(user_id, redis)?;
        if user_room.set_ready_time(redis)? {
            Room::write_data(&user_room.id, redis, client, ch_url).await?;

            let idx = Room::prepare_for_next_question(&user_room.id, redis)?;
            let users: Vec<i32> = redis.hget(Room::key(&user_room.id), &["creator_id", "visitor_id"])?;
            let pack: String = redis.hget(Room::key(&user_room.id), "pack")?;
            send_question_messages([users[0], users[1]], &pack, idx, redis, client, url, &user_room.id, ch_url).await?;

            Ok(None)
        } else {
            Ok(Some(OutgoingKeyboardMessage::with_text(user_id, Messages::WAITING_FOR_PARTNER_EVAL)))
        }
    }
}