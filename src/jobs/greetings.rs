use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio_cron_scheduler::Job;
use tracing::instrument;

use crate::types::MyResult;
use crate::{settings::Settings, stickers::send_sticker};

#[instrument(skip_all)]
pub async fn greeting_morning(bot: Bot, settings: Settings, chat_id: ChatId) -> MyResult<Job> {
    let job = Job::new_async("0 30 23 * * * *", move |_uuid, _lock| {
        let bot = bot.clone();
        let settings = settings.clone();
        Box::pin(async move {
            send_sticker(&bot, &chat_id, settings.stickers.hello)
                .await
                .unwrap();
            bot.send_message(chat_id, "~ HELLO GOOD MORNING!! ~ 🐢")
                .await
                .unwrap();
        })
    })
    .expect("error initiating morning greeting");
    Ok(job)
}

#[instrument(skip_all)]
pub async fn greeting_night(bot: Bot, settings: Settings, chat_id: ChatId) -> MyResult<Job> {
    let job = Job::new_async("0 00 15 * * * *", move |_uuid, _lock| {
        let bot = bot.clone();
        let settings = settings.clone();
        Box::pin(async move {
            send_sticker(&bot, &chat_id, settings.stickers.sleep.to_string())
                .await
                .unwrap();
            bot.send_message(
                chat_id,
                "It has been a tiring day 😩.\nGoodnight 😪.\n~ See you in dreamland ~ 🐢",
            )
            .await
            .unwrap();
        })
    })
    .expect("error initiating night greeting");

    Ok(job)
}
