use crate::telegram::structures::CallbackData;

pub(crate) fn create_tg_url(bot_token: &str, method: &str) -> String {
    format!("https://api.telegram.org/bot{}/{}", bot_token, method)
}

#[derive(Debug)]
pub enum UpdateType {
    Start,
    Help,
    JoinExisting,
    Create,
    Callback(i64, i32, CallbackData, String),
    NewRoom,
    InsertId,
    WaitingForOther,
    WaitingForResults,
    UnknownCommand,
    Other,
    Error,
}

#[derive(Debug)]
pub(crate) enum CallbackMessageType {
    Importance,
    Evaluation,
    Error
}
