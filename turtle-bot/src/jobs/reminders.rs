use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio_cron_scheduler::Job;
use uuid::Uuid;

use super::CronJobError;

#[derive(Clone)]
struct Reminder {
    id: i32,
    target: i64,
    message: String,
    username: String,
    due: DateTime<Utc>,
}

struct RemindMetadata {
    id: i32,
    job_id: Uuid,
}
pub async fn get_reminders(bot: &Bot, pool: &PgPool) -> Result<Vec<Job>, CronJobError> {
    let reminders: Vec<Reminder> = sqlx::query_as!(
        Reminder,
        "
        SELECT id, target, message, username, due
        FROM jobs_one_off WHERE completed = false 
        AND due IS NOT NULL
        AND due >= CURRENT_TIMESTAMP"
    )
    .fetch_all(pool)
    .await?;
    if reminders.is_empty() {
        return Ok(Vec::new());
    }

    let now = Utc::now();
    let mut job_vec: Vec<Job> = Vec::new();
    let mut job_metadata: Vec<RemindMetadata> = Vec::new();

    for remind in reminders {
        let time_delta_secs = (remind.due - now).num_seconds();
        let seconds = u64::from_ne_bytes(time_delta_secs.to_ne_bytes());

        let remindx = remind.clone();
        let botx = bot.clone();
        let poolx = pool.clone();
        let job = Job::new_one_shot_async(Duration::from_secs(seconds), move |_, _| {
            let bot = botx.clone();
            let remind = remindx.clone();
            let pool = poolx.clone();
            Box::pin(async move {
                let text = format!(
                    r"From: @{}

{}",
                    remind.username, remind.message
                );
                if let Err(e) = bot.send_message(ChatId(remind.target), text).await {
                    tracing::error!("error sending one-off-job {e:#?}");
                }

                if let Err(e) = sqlx::query!(
                    r#"UPDATE jobs_one_off SET completed = $1 WHERE target = $2 and due = $3 and username = $4 "#,
                    true, remind.target, remind.due, remind.username)
                .execute(&pool)
                .await
                {
                    tracing::error!("error updating completed one_off_job in database: {e:#?}");
                }
            })
        })?;

        let job_id = job.guid();

        job_metadata.push(RemindMetadata {
            job_id,
            id: remind.id,
        });
        job_vec.push(job);
    }

    for job_meta in job_metadata {
        tokio::spawn(update_job(job_meta, pool.clone()));
    }
    Ok(job_vec)
}
/// Update database with the new `job_id/Uuid`.
#[tracing::instrument(skip_all)]
async fn update_job(data: RemindMetadata, pool: PgPool) {
    if let Err(e) = sqlx::query!(
        "UPDATE jobs_cron set job_id=$1 WHERE id=$2",
        data.job_id,
        data.id
    )
    .execute(&pool)
    .await
    {
        tracing::error!(error = %e);
    }
}
