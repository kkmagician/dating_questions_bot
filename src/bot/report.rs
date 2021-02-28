use crate::ternary;
use crate::bot::room::Role;

use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReportData {
    creator_total: i32,
    visitor_total: i32,
    share_positive_creator: f32,
    share_positive_visitor: f32,
    creator_avg: f32,
    visitor_avg: f32
}

impl ReportData {
    pub async fn get(room_id: &String, client: &Client, ch_url: &String)
        -> Result<ReportData, reqwest::Error> {
        let res = client.post(ch_url)
            .body(ReportData::generate_request(room_id))
            .send().await?
            .json::<ReportData>()
            .await;

        res
    }

    pub(crate) fn generate_report(&self, role: &String) -> String {
        let your_total: i32 = ternary!(role == Role::CREATOR, self.creator_total, self.visitor_total);
        let other_total: i32 = ternary!(role == Role::CREATOR, self.visitor_total, self.creator_total);
        let other_share_positive: f32 = ternary!(role == Role::CREATOR, self.share_positive_visitor, self.share_positive_creator);
        let your_share_positive: f32 = ternary!(role == Role::CREATOR, self.share_positive_creator, self.share_positive_visitor);
        let other_avg: f32 = ternary!(role == Role::CREATOR, self.visitor_avg, self.creator_avg);
        let your_avg: f32 = ternary!(role == Role::CREATOR, self.creator_avg, self.visitor_avg);

        format!(r#"✨<b>Твой отчет:</b>
🤗Ты оценил партнера на <i>{your_total}</i>, а он тебя – на <i>{other_total}</i>.

💥Позитивную оценку получили <i>{other_share_positive:.1}%</i> твоих ответов, а ты положительно оценил <i>{your_share_positive:.1}%</i> ответов партнера.

🃏Средняя оценка твоих ответов: <i>{other_avg:.2}</i>, а ты оценивал ответы партнера в среднем на <i>{your_avg:.1}</i>."#,
                your_total = your_total, other_total = other_total,
                other_share_positive = other_share_positive,
                your_share_positive = your_share_positive,
                other_avg = other_avg, your_avg = your_avg
        )
    }

    fn generate_request(room_id: &String) -> String {
        format!(r#"
            select
                toUInt16(count()) as total_questions,
                countIf(creator_score > 0) / total_questions * 100 as share_positive_creator,
                countIf(visitor_score > 0) / total_questions * 100 as share_positive_visitor,
                toInt32(sum(creator_score)) as creator_total,
                toInt32(sum(visitor_score)) as visitor_total,
                avg(creator_score) as creator_avg,
                avg(visitor_score) as visitor_avg
            from (
                select
                    creator_importance * (creator_evaluation - 2) as creator_score,
                    visitor_importance * (visitor_evaluation - 2) as visitor_score
                from tg_room_bot
                where room_id = '{}' and [creator_score, visitor_score] != [0, 0]
            ) format JSONEachRow"#, room_id
        )
    }
}