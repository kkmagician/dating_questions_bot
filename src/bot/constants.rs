pub(crate) const EMOJIS: [&str; 5] = ["😡", "🙁", "😐", "😊", "️😀"];

pub(crate) struct RedisKeys;
impl RedisKeys {
    pub const PACKS: &'static str = "packs";
    pub const LATEST_MESSAGE: &'static str = "latest_message";
}

pub struct Messages;
impl Messages {
    pub const WELCOME: &'static str = "Привет! Рад видеть тебя.";
    pub const ERROR: &'static str = "Ошибка";
    pub const WAITING_FOR_PARTNER: &'static str = "Ожидание партнера";
    pub const WAITING_FOR_PARTNER_EVAL: &'static str = "Ожидание оценок партнера";
    pub const INSERT_ROOM_ID: &'static str = "Введи ID комнаты";
    pub const WRONG_ROOM_ID: &'static str = "Неверный ID комнаты, попробуй еще";
    pub const CHOOSE_PACK: &'static str = "Выбери набор";
    pub const READY_FOR_NEXT: &'static str = "Скажи, когда будешь готов продолжить";
    pub const EVALUATING_RESULTS: &'static str = "Это был последний вопрос! Подожди, пока подвожу итоги...";
    pub const WAIT_A_MOMENT: &'static str = "Подожди минутку...";
    pub const ANSWER_IMPORTANCE: &'static str = "Насколько тебе важен ответ?";
    pub const ANSWER_EVALUATION: &'static str = "Как оцениваешь ответ?";
}

pub struct Keys;
impl Keys {
    pub const WELCOME: [&'static str; 2] = ["Вступить", "Сoздать"];
    pub const READY: &'static str = "Готов!";
    pub fn welcome() -> Vec<Vec<String>> {
        vec![Keys::WELCOME.to_vec().iter().map(|&x| String::from(x)).collect()]
    }
}