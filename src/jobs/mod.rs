use crate::{
    settings::{Environment, Settings},
    types::MyResult,
};
use teloxide::{types::ChatId, Bot};
use tokio_cron_scheduler::JobScheduler;

mod greetings;

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

    let job_morning = greetings::greeting_morning(bot.clone(), settings.clone(), chat_id).await?;
    sched.add(job_morning).await?;

    let job_night = greetings::greeting_night(bot.clone(), settings.clone(), chat_id).await?;
    sched.add(job_night).await?;

    load_db_jobs().await;

    sched.shutdown_on_ctrl_c();

    sched.start().await?;
    Ok(sched)
}

async fn load_db_jobs() {
    // TODO load db jobs
}
