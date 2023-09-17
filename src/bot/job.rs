use crate::{
    settings::{Environment, Settings},
    stickers::send_sticker,
    types::MyResult,
};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::instrument;
pub async fn start_jobs(
    bot: &Bot,
    settings: &Settings,
    env: Environment,
) -> MyResult<JobScheduler> {
    let sched = JobScheduler::new().await?;

    let chat_id = match env {
        Environment::Local => ChatId(-1001838253386),
        Environment::Production => ChatId(-907471997),
    };

    let job_morning = greeting_morning(bot.clone(), settings.clone(), chat_id).await?;
    sched.add(job_morning).await?;

    let job_night = greeting_night(bot.clone(), settings.clone(), chat_id).await?;
    sched.add(job_night).await?;

    load_db_jobs().await;

    sched.shutdown_on_ctrl_c();

    sched.start().await?;
    Ok(sched)
}

async fn load_db_jobs() {
    // TODO load db jobs
}

#[instrument(skip_all)]
async fn greeting_morning(bot: Bot, settings: Settings, chat_id: ChatId) -> MyResult<Job> {
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
async fn greeting_night(bot: Bot, settings: Settings, chat_id: ChatId) -> MyResult<Job> {
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
