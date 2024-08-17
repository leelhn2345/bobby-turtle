mod greetings;
mod reminders;

use gaia::stickers::Stickers;
use sqlx::PgPool;
use teloxide::Bot;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::jobs::{greetings::get_greetings, reminders::get_reminders};

#[derive(thiserror::Error, Debug)]
pub enum CronJobError {
    #[error(transparent)]
    CronScheduler(#[from] JobSchedulerError),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

#[tracing::instrument(skip_all)]
pub async fn add_job(scheduler: JobScheduler, job: Job) {
    if let Err(e) = scheduler.add(job).await {
        tracing::error!(%e);
    }
}

#[tracing::instrument(skip_all)]
pub async fn init_scheduler(
    bot: &Bot,
    stickers: &Stickers,
    pool: &PgPool,
) -> Result<JobScheduler, CronJobError> {
    let scheduler = JobScheduler::new().await?;
    let mut greeting_jobs = get_greetings(bot, stickers, pool).await.map_err(|e| {
        tracing::error!(error = %e);
        e
    })?;
    let mut remind_jobs = get_reminders(bot, pool).await.map_err(|e| {
        tracing::error!(error = %e);
        e
    })?;

    greeting_jobs.append(&mut remind_jobs);

    for job in greeting_jobs {
        tokio::spawn(add_job(scheduler.clone(), job));
    }

    scheduler.shutdown_on_ctrl_c();
    scheduler.start().await?;
    tracing::debug!("scheduler started");
    Ok(scheduler)
}

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
