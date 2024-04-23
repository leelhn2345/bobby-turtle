use std::sync::{Arc, Mutex};

use chrono_tz::Tz;
use sqlx::{migrate::Migrate, PgPool, QueryBuilder};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio_cron_scheduler::Job;
use uuid::Uuid;

use crate::{bot::send_sticker, settings::stickers::Stickers};

use super::CronJobError;

struct Greeting {
    id: i32,
    target: i64,
    cron_str: String,
    message: String,
}

struct JobMetadata {
    id: i32,
    guid: Uuid,
}

struct GreetingJob {
    job: Vec<Job>,
    metadata: Vec<JobMetadata>,
}

#[tracing::instrument(skip_all)]
pub async fn get_greetings(
    bot: &Bot,
    stickers: &Stickers,
    pool: &PgPool,
) -> Result<Vec<Job>, CronJobError> {
    let mut greeting_jobs: Vec<Job> = Vec::new();
    let mut metadata_jobs: Vec<JobMetadata> = Vec::new();

    let mut night_jobs = night_greeting(bot, stickers, pool).await?;
    greeting_jobs.append(&mut night_jobs.job);
    metadata_jobs.append(&mut night_jobs.metadata);

    let mut morning_jobs = morning_greeting(bot, stickers, pool).await?;
    greeting_jobs.append(&mut morning_jobs.job);
    metadata_jobs.append(&mut morning_jobs.metadata);

    for data in metadata_jobs {
        tokio::spawn(update_job(data, pool.clone()));
    }
    Ok(greeting_jobs)
}

/// Update database with the new `job_id/Uuid`.
#[tracing::instrument(skip_all)]
async fn update_job(data: JobMetadata, pool: PgPool) {
    if let Err(e) = sqlx::query!(
        "UPDATE jobs_cron set job_id=$1 WHERE id=$2",
        data.guid,
        data.id
    )
    .execute(&pool)
    .await
    {
        tracing::error!(error = %e);
    }
}

#[tracing::instrument(skip_all)]
async fn send_greeting(bot: Bot, msg_id: i64, msg: String, sticker: String) {
    if let Err(e) = send_sticker(&bot, &ChatId(msg_id), sticker).await {
        tracing::error!(error = %e);
    };
    if let Err(e) = bot.send_message(ChatId(msg_id), msg).await {
        tracing::error!(error = %e);
    }
}

#[tracing::instrument(skip_all)]
async fn morning_greeting(
    bot: &Bot,
    stickers: &Stickers,
    pool: &PgPool,
) -> Result<GreetingJob, CronJobError> {
    let mut job_vec: Vec<Job> = Vec::new();
    let mut job_metadata: Vec<JobMetadata> = Vec::new();

    let jobs_in_db: Vec<Greeting> = sqlx::query_as!(
        Greeting,
        "
        SELECT id, target, cron_str, message from jobs_cron
        WHERE type = 'morning-greeting'
        "
    )
    .fetch_all(pool)
    .await?;

    if jobs_in_db.is_empty() {
        return Ok(GreetingJob {
            job: job_vec,
            metadata: job_metadata,
        });
    };

    for cron_job in jobs_in_db {
        let bot = bot.clone();
        let stickers = stickers.to_owned();
        let job = Job::new_async_tz(cron_job.cron_str.as_str(), Tz::Singapore, move |_, _| {
            let bot = bot.clone();
            let sleep_sticker = stickers.hello.clone();
            let msg = cron_job.message.clone();

            Box::pin(send_greeting(bot, cron_job.target, msg, sleep_sticker))
        });

        match job {
            Ok(x) => {
                job_metadata.push(JobMetadata {
                    id: cron_job.id,
                    guid: x.guid(),
                });
                job_vec.push(x);
            }

            Err(e) => tracing::error!(error = %e),
        }
    }

    Ok(GreetingJob {
        job: job_vec,
        metadata: job_metadata,
    })
}

#[tracing::instrument(skip_all)]
async fn night_greeting(
    bot: &Bot,
    stickers: &Stickers,
    pool: &PgPool,
) -> Result<GreetingJob, CronJobError> {
    let mut job_vec: Vec<Job> = Vec::new();
    let mut job_metadata: Vec<JobMetadata> = Vec::new();

    let jobs_in_db: Vec<Greeting> = sqlx::query_as!(
        Greeting,
        "
        SELECT id, target, cron_str, message from jobs_cron
        WHERE type = 'night-greeting'
        "
    )
    .fetch_all(pool)
    .await?;

    if jobs_in_db.is_empty() {
        return Ok(GreetingJob {
            job: job_vec,
            metadata: job_metadata,
        });
    };

    for cron_job in jobs_in_db {
        let bot = bot.clone();
        let stickers = stickers.to_owned();
        let job = Job::new_async_tz(cron_job.cron_str.as_str(), Tz::Singapore, move |_, _| {
            let bot = bot.clone();
            let sleep_sticker = stickers.sleep.clone();
            let msg = cron_job.message.clone();

            Box::pin(send_greeting(bot, cron_job.target, msg, sleep_sticker))
        });

        match job {
            Ok(x) => {
                job_metadata.push(JobMetadata {
                    id: cron_job.id,
                    guid: x.guid(),
                });
                job_vec.push(x);
            }

            Err(e) => tracing::error!(error = %e),
        }
    }

    Ok(GreetingJob {
        job: job_vec,
        metadata: job_metadata,
    })
}
