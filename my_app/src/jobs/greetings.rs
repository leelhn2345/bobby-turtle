use chrono_tz::Tz;
use my_app::Stickers;
use my_bot::sticker::send_sticker;
use sqlx::PgPool;
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio_cron_scheduler::Job;
use uuid::Uuid;

use super::CronJobType;

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
pub async fn get_greetings(bot: &Bot, stickers: &Stickers, pool: &PgPool) -> Vec<Job> {
    let mut greeting_jobs: Vec<Job> = Vec::new();
    let mut metadata_jobs: Vec<JobMetadata> = Vec::new();

    let mut night_jobs = get_greetings_from_db(
        bot,
        &stickers.sleep,
        pool,
        CronJobType::NightGreeting.as_str(),
    )
    .await;
    greeting_jobs.append(&mut night_jobs.job);
    metadata_jobs.append(&mut night_jobs.metadata);

    let mut morning_jobs = get_greetings_from_db(
        bot,
        &stickers.hello,
        pool,
        CronJobType::MorningGreeting.as_str(),
    )
    .await;
    greeting_jobs.append(&mut morning_jobs.job);
    metadata_jobs.append(&mut morning_jobs.metadata);

    for data in metadata_jobs {
        tokio::spawn(update_job(data, pool.clone()));
    }
    greeting_jobs
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
async fn get_greetings_from_db(
    bot: &Bot,
    sticker: &String,
    pool: &PgPool,
    job_type: &str,
) -> GreetingJob {
    let mut job_vec: Vec<Job> = Vec::new();
    let mut job_metadata: Vec<JobMetadata> = Vec::new();

    let jobs_in_db: Vec<Greeting> = sqlx::query_as!(
        Greeting,
        "
        SELECT id, target, cron_str, message from jobs_cron
        WHERE type = $1
        ",
        job_type
    )
    .fetch_all(pool)
    .await
    .unwrap();

    if jobs_in_db.is_empty() {
        return GreetingJob {
            job: job_vec,
            metadata: job_metadata,
        };
    };

    for cron_job in jobs_in_db {
        let bot = bot.clone();
        let sticker = sticker.to_owned();
        let job = Job::new_async_tz(cron_job.cron_str.as_str(), Tz::Singapore, move |_, _| {
            let bot = bot.clone();
            let sticker = sticker.clone();
            let msg = cron_job.message.clone();

            Box::pin(send_greeting(bot, ChatId(cron_job.target), msg, sticker))
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

    GreetingJob {
        job: job_vec,
        metadata: job_metadata,
    }
}

#[tracing::instrument(skip_all)]
async fn send_greeting(bot: Bot, chat_id: ChatId, msg: String, sticker: String) {
    if let Err(e) = send_sticker(bot.clone(), chat_id, sticker).await {
        tracing::error!(error = %e);
    };
    if let Err(e) = bot.send_message(chat_id, msg).await {
        tracing::error!(error = %e);
    }
}
