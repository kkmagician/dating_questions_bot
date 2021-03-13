pub(crate) const IMPORTANCE_EMOJIS: [&str; 5] = [" 0️", "✔️", "❗", "‼️", "️🔥"];
pub(crate) const EVALUATION_EMOJIS: [&str; 5] = ["😡", "🙁", "😐", "😊", "️😀"];

pub(crate) struct RedisKeys;
impl RedisKeys {
    pub const PACKS: &'static str = "packs";
    pub const LATEST_MESSAGE: &'static str = "latest_message";
}

pub struct Messages;
impl Messages {
    pub const WELCOME: &'static str = r#"👋Привет! Рад видеть тебя.
Нажми "Создать", чтобы выбрать набор вопросов и запустить комнату.
Нажми "Вступить", чтобы вставить получанный от партнера ID комнаты и начать общение.
"#;
    pub const WAITING_FOR_PARTNER: &'static str = "Ждем, пока партнер зайдет в комнату.";
    pub const WAITING_FOR_PARTNER_EVAL: &'static str = "Ожидание оценок партнера";
    pub const INSERT_ROOM_ID: &'static str = "Введи ID комнаты";
    pub const NO_ROOM_ID_IN_MESSAGE: &'static str = "Не могу найти ID в тексте сообщения.";
    pub const WRONG_ROOM_ID: &'static str = "Неверный ID комнаты, попробуй еще.";
    pub const CHOOSE_PACK: &'static str = "Выбери набор";
    pub const READY_FOR_NEXT: &'static str = "Скажи, когда будешь готов продолжить";
    pub const EVALUATING_RESULTS: &'static str = "Это был последний вопрос! Подожди, пока подвожу итоги...";
    pub const WAIT_A_MOMENT: &'static str = "Подожди минутку...";
    pub const ANSWER_IMPORTANCE: &'static str = "Насколько тебе важен ответ?";
    pub const ANSWER_EVALUATION: &'static str = "Как тебе ответ партнера?";
    pub const HELP: &'static str = r#"Бот, который присылает вопросы для обсуждения.

Нажми "Создать", чтобы выбрать набор вопросов и запустить комнату. Бот пришлет ID комнаты: его нужно отправить партнеру.
Нажми "Вступить", чтобы вставить получанный от партнера ID комнаты и начать общение.

Бот будет присылать вопросы для обсуждения по одному. Общайтесь, оценивайте важность ответа партнера и то, насколько ответ понравился.
Когда будут оценены все вопросы, бот пришлет маленький отчет, в котором расскажет, как вы оценили друг друга.
"#;

    pub const ERROR: &'static str = "Ошибка, попробуй ещё.";
    pub const ERROR_PACK_DOES_NOT_EXIST: &'static str = "Такого набора не существует, попробуй выбрать кнопкой.";
    pub const ERROR_INTERNAL: &'static str = "Ошибка бота, попробуй ещё раз немного позже.";
    pub const ERROR_UNKNOWN_COMMAND: &'static str = "Неизвестная команда бота. Попробуй выбрать из предложенного списка.";
}

pub struct Keys;
impl Keys {
    pub const CREATE: &'static str = "🧩Сoздать";
    pub const JOIN: &'static str = "🎟Вступить";
    pub const READY: &'static str = "Готов!";

    pub fn welcome() -> Vec<Vec<String>> {
        vec![[Keys::CREATE, Keys::JOIN].iter().map(|&x| String::from(x)).collect()]
    }
}
