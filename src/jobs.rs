mod greetings;
use sqlx::PgPool;
use teloxide::Bot;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::{jobs::greetings::get_greetings, settings::stickers::Stickers};

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
    let greeting_jobs = get_greetings(bot, stickers, pool).await.map_err(|e| {
        tracing::error!(error = %e);
        e
    })?;
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use chrono::{DateTime, Local, NaiveDateTime};
    use tokio_cron_scheduler::{Job, JobScheduler};
    use tracing::{info, Level};
    use tracing_subscriber::FmtSubscriber;

    // Needs multi_thread to test, otherwise it hangs on scheduler.add()
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_schedule() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .without_time()
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Setting default subscriber failed");

        info!("Create scheduler");
        let scheduler = JobScheduler::new().await.unwrap();
        info!("Add job");
        scheduler
            .add(
                Job::new_one_shot(Duration::from_secs(10), |_uuid, _l| {
                    println!("{:?} I'm only run once", chrono::Local::now());
                })
                .unwrap(),
            )
            .await
            .expect("Should be able to add a job");
        let job = Job::new_async("*/1  * * * * *", |_, _| {
            Box::pin(async {
                info!("Run every seconds");
            })
        })
        .unwrap();

        scheduler.add(job).await.expect("shoudl be able to add job");

        scheduler.start().await.unwrap();

        tokio::time::sleep(core::time::Duration::from_secs(20)).await;
    }

    #[test]
    fn datetime_diff() {
        let predetermined_datetime = DateTime::parse_from_rfc3339("2024-04-23T21:00:00+08:00")
            .expect("Invalid datetime format")
            .with_timezone(&Local);

        // Get the current datetime
        let current_datetime = Local::now();

        // Calculate the duration from the predetermined datetime to the current time
        let duration_from_now = predetermined_datetime
            .signed_duration_since(current_datetime)
            .num_minutes();

        println!("Duration from predetermined datetime to now: {duration_from_now:?}");
    }

    #[test]
    fn parse_naivedatetime() {
        let naive_datetime =
            NaiveDateTime::parse_from_str("2024-04-23 12:00:00", "%Y-%m-%d %H:%M:%S")
                .expect("Invalid datetime format");

        println!("NaiveDateTime: {naive_datetime}");

        // Manipulate NaiveDateTime
        let modified_datetime = naive_datetime + chrono::Duration::hours(3);
        println!("Modified NaiveDateTime: {modified_datetime}");
    }
}
