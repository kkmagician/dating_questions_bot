use crate::tools::*;
use crate::telegram::messages::send_question_messages;

use serde::Deserialize;
use redis::{Commands, Connection};
use reqwest::Client;
use std::collections::HashMap;
use std::borrow::Borrow;

pub struct Role;
impl Role {
    fn opposite(role: &String) -> String {
        if role == Role::CREATOR {
            Role::VISITOR.to_string()
        } else {
            Role::CREATOR.to_string()
        }
    }

    pub fn get(user_id: i32, redis: &mut Connection)
           -> Result<String, redis::RedisError> {
        let role: String = redis.hget(format!("user:{}:room", user_id), "role")?;
        Ok(role)
    }

    pub const CREATOR: &'static str = "creator";
    pub const VISITOR: &'static str = "visitor";
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Room {
    room_id: String,
    creator_id: i32,
    visitor_id: Option<i32>,
    pack: String,
    created_at: u64
}

impl Room {
    fn key(room_id: &String) -> String {
        format!("room:{}", room_id)
    }

    pub(crate) fn create(user_id: i32, pack: &String, redis: &mut redis::Connection)
                         -> String {
        let room_id = random_id();

        let _: redis::RedisResult<()> = redis.hset_multiple(
            Room::key(&room_id),
            &[
                ("room_id", room_id.to_string()),
                ("creator_id", user_id.to_string()),
                ("pack", pack.to_string()),
                ("created_at", current_time().to_string()),
                ("idx", "0".to_string())
            ]
        );

        room_id
    }

    pub(crate) fn enter(room_id: &String, user_id: i32, redis: &mut redis::Connection)
        -> Result<(), redis::RedisError> {
        redis.hset(Room::key(room_id), "visitor_id", user_id.to_string())?;
        Ok(())
    }

    pub(crate) fn prepare_for_next_question(room_id: &String, redis: &mut redis::Connection)
                                            -> Result<u16, redis::RedisError> {
        let key = Room::key(room_id);
        redis.hdel(
            &key,
            &[
                "visitor_importance",
                "visitor_evaluation",
                "visitor_ready_at",
                "creator_importance",
                "creator_evaluation",
                "creator_ready_at"
            ]
        )?;
        let new_idx: u16 = redis.hincr(&key, "idx", 1)?;
        Ok(new_idx)
    }

    pub(crate) async fn start(
        room_id: &String,
        redis: &mut redis::Connection,
        client: &Client,
        url: &str,
        ch_url: &String
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (creator_id, visitor_id, pack): (i32, i32, String) = redis.hget(
            Room::key(room_id),
            &["creator_id", "visitor_id", "pack"]
        )?;

        Context::set_context(creator_id, Context::IN_ROOM, redis)?;
        Context::set_context(visitor_id, Context::IN_ROOM, redis)?;

        redis.hset_multiple(
            format!("user:{}:room", creator_id),
            &[
                ("id", room_id),
                ("role", &Role::CREATOR.to_string())
            ]
        )?;

        redis.hset_multiple(
            format!("user:{}:room", visitor_id),
            &[
                ("id", room_id),
                ("role", &Role::VISITOR.to_string())
            ]
        )?;

        send_question_messages(
            [creator_id, visitor_id],
            &pack, 0, redis, client, url, room_id, ch_url
        ).await?;

        Ok(())
    }

    pub(crate) async fn write_data(
        room_id: &String,
        redis: &mut redis::Connection,
        client: &Client,
        ch_url: &String
    ) -> Result<(), Box<dyn std::error::Error>> {
        let room: HashMap<String, String> = redis.hgetall(Room::key(room_id))?;

        let creator_id: i32 = get_parse_string_value(&room, "creator_id", 0);
        let visitor_id: i32 = get_parse_string_value(&room, "visitor_id", 0);
        let created_at: i32 = get_parse_string_value(&room, "created_at", 0);
        let idx: u16 = get_parse_string_value(&room, "idx", 0);
        let creator_importance: i8 = get_parse_string_value(&room, "creator_importance", 0);
        let creator_evaluation: i8 = get_parse_string_value(&room, "creator_evaluation", 0);
        let visitor_importance: i8 = get_parse_string_value(&room, "visitor_importance", 0);
        let visitor_evaluation: i8 = get_parse_string_value(&room, "visitor_evaluation", 0);
        let creator_ready_at: i32 = get_parse_string_value(&room, "creator_ready_at", 0);
        let visitor_ready_at: i32 = get_parse_string_value(&room, "visitor_ready_at", 0);

        let query = format!(r#"
            INSERT INTO tg_room_bot (room_id, creator_id, visitor_id, pack, created_at, idx, creator_importance, creator_evaluation, visitor_importance, visitor_evaluation, creator_ready_at, visitor_ready_at)
            VALUES ('{room_id}', {creator_id}, {visitor_id}, '{pack}', {created_at}, {idx}, {creator_importance}, {creator_evaluation}, {visitor_importance}, {visitor_evaluation}, {creator_ready_at}, {visitor_ready_at})
        "#, room_id = room_id,
                            creator_id = creator_id,
                            visitor_id = visitor_id,
                            pack = room.get("pack").unwrap_or("".to_string().borrow()),
                            created_at = created_at,
                            idx = idx,
                            creator_importance = creator_importance,
                            creator_evaluation = creator_evaluation,
                            visitor_importance = visitor_importance,
                            visitor_evaluation = visitor_evaluation,
                            creator_ready_at = creator_ready_at,
                            visitor_ready_at = visitor_ready_at
        );

        client.post(ch_url).body(query).send().await?;
        Ok(())
    }

    pub(crate) fn clear(room_id: &String, redis: &mut redis::Connection)
        -> redis::RedisResult<()> {
        redis.del(Room::key(room_id))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UserRoom {
    pub(crate) id: String,
    pub(crate) role: String
}

impl UserRoom {
    pub fn get(user_id: i32, redis: &mut redis::Connection)
           -> Result<UserRoom, redis::RedisError> {
        let role: HashMap<String, String> = redis.hgetall(format!("user:{}:room", user_id))?;

        Ok(UserRoom {
            id: (&role.get("id").unwrap()).to_string(),
            role: (&role.get("role").unwrap()).to_string()
        })
    }

    pub(crate) fn set_ready_time(&self, redis: &mut redis::Connection)
                                 -> Result<bool, redis::RedisError> {
        let key = format!("room:{}", self.id);
        let role_field = format!("{}_ready_at", self.role);
        let opposite_role_field = format!("{}_ready_at", Role::opposite(&self.role));
        let is_already_set: bool = redis.hexists(&key, &role_field)?;

        if !is_already_set {
            redis.hset(&key, &role_field, current_time().to_string())?
        }

        if redis.hexists(&key, &opposite_role_field)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub struct Context;
impl Context {
    pub fn get(user_id: i32, redis: &mut redis::Connection)
           -> redis::RedisResult<String> {
        redis.get(Context::key(user_id))
    }

    pub(crate) fn set_context(user_id: i32, context: &'static str, redis: &mut redis::Connection)
                              -> redis::RedisResult<()> {
        redis.set(Context::key(user_id), context)
    }

    pub(crate) fn reset(user_id: i32, redis: &mut redis::Connection) -> redis::RedisResult<()> {
        redis.del(Context::key(user_id))
    }

    pub fn key(user_id: i32) -> String {
        format!("user:{}:context", user_id)
    }

    pub const SELECT_PACK: &'static str = "SELECT_PACK";
    pub const INSERT_ID: &'static str = "INSERT_ID";
    pub const WAITING_FOR_PARTNER: &'static str = "WAITING_FOR_PARTNER";
    pub const WAITING_FOR_ANSWER: &'static str = "WAITING_FOR_ANSWER";
    pub const IN_ROOM: &'static str = "IN_ROOM";
    pub const WAITING_FOR_RESULTS: &'static str = "WAITING_FOR_RESULTS";
}


