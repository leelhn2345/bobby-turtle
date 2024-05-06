use my_app::Stickers;
use sqlx::PgPool;
use teloxide::Bot;
use tokio_cron_scheduler::{Job, JobScheduler};

use self::{greetings::get_greetings, reminders::get_reminders};

mod greetings;
mod reminders;

pub enum CronJobType {
    MorningGreeting,
    NightGreeting,
}

impl CronJobType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MorningGreeting => "morning-greeting",
            Self::NightGreeting => "night-greeting",
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn add_job(scheduler: JobScheduler, job: Job) {
    if let Err(e) = scheduler.add(job).await {
        tracing::error!(%e);
    }
}

#[tracing::instrument(skip_all)]
pub async fn init_scheduler(bot: &Bot, stickers: &Stickers, pool: &PgPool) -> JobScheduler {
    let scheduler = JobScheduler::new().await.expect("cannot init scheduler");

    let mut greeting_jobs = get_greetings(bot, stickers, pool).await;

    let mut remind_jobs = get_reminders(bot, pool).await;

    greeting_jobs.append(&mut remind_jobs);

    for job in greeting_jobs {
        tokio::spawn(add_job(scheduler.clone(), job));
    }

    scheduler.shutdown_on_ctrl_c();
    scheduler.start().await.expect("can't start scheduler");

    scheduler
}
