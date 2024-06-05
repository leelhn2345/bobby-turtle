use std::time::Duration;

use anyhow::bail;
use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{
        CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
    },
    Bot,
};
use tokio_cron_scheduler::{Job, JobScheduler};

use super::{expired_callback_msg, time_check, CallbackPage, CallbackState};

const JOB_TEXT_BACK: &str = "Back";
const JOB_TEXT_CONFIRM: &str = "Confirm";
const CHANGE_TIME: &str = "Change Time";

pub async fn remind_text_page(
    bot: Bot,
    chat_id: ChatId,
    msg_id: MessageId,
    chosen_datetime: DateTime<Tz>,
) -> anyhow::Result<()> {
    let chosen_year = chosen_datetime.year();
    let chosen_month = chosen_datetime.month();
    let chosen_day = chosen_datetime.day();
    let chosen_hour = chosen_datetime.hour();
    let chosen_minute = chosen_datetime.minute();

    let text = format!(
        r"You have chosen:

year: {chosen_year}
month: {chosen_month} 
day: {chosen_day}
hour: {chosen_hour}
minute: {chosen_minute}

What is it that you want me to remind you of?
Say it in your next message. 🐢"
    );

    bot.edit_message_text(chat_id, msg_id, text)
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Back", CHANGE_TIME),
        ]]))
        .await?;
    Ok(())
}

pub async fn confirm_reminder_text(
    bot: Bot,
    msg: Message,
    chosen_datetime: DateTime<Tz>,
    callback: CallbackState,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        bail!("no text")
    };

    if text.is_empty() {
        bail!("empty text")
    }
    let chosen_year = chosen_datetime.year();
    let chosen_month = chosen_datetime.month();
    let chosen_day = chosen_datetime.day();
    let chosen_hour = chosen_datetime.hour();
    let chosen_minute = chosen_datetime.minute();

    let job_msg = format!(
        r"You have chosen:

year: {chosen_year}
month: {chosen_month} 
day: {chosen_day}
hour: {chosen_hour}
minute: {chosen_minute}

text:
{text}"
    );

    callback
        .update(CallbackPage::ConfirmOneOffJob {
            date_time: chosen_datetime,
            msg_text: text.to_string(),
        })
        .await?;

    let keyboard = job_text_keyboard();

    bot.send_message(msg.chat.id, job_msg)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
fn job_text_keyboard() -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![vec![
        InlineKeyboardButton::callback(JOB_TEXT_BACK, JOB_TEXT_BACK),
        InlineKeyboardButton::callback(JOB_TEXT_CONFIRM, JOB_TEXT_CONFIRM),
    ]];

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn remind_text_callback(
    bot: Bot,
    q: CallbackQuery,
    p: CallbackState,
    (date_time, msg_text): (DateTime<Tz>, String),
    pool: PgPool,
    sched: JobScheduler,
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id).await?;

    let Some(data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        bail!("no query callback data")
    };
    let Some(Message {
        id: msg_id, chat, ..
    }) = q.message
    else {
        tracing::error!("no message data from telegram");
        bail!("no telegram message data")
    };

    let Some(username) = q.from.username else {
        tracing::warn!("no username given for this reminder text");
        bail!("wtf");
    };

    let now = Utc::now().with_timezone(&Tz::Singapore);

    time_check(&bot, chat.id, date_time, now).await?;

    match data.as_ref() {
        JOB_TEXT_BACK => {
            p.update(CallbackPage::ConfirmDateTime { date_time })
                .await?;

            remind_text_page(bot, chat.id, msg_id, date_time).await?;
        }
        JOB_TEXT_CONFIRM => {
            let time_delta = date_time - now;
            let time_delta_secs = time_delta.num_seconds();
            let seconds = u64::from_le_bytes(time_delta_secs.to_le_bytes());

            let bot_clone = bot.clone();
            let text_clone = msg_text.clone();
            let pool_clone = pool.clone();
            let username_clone = username.clone();

            let job = Job::new_one_shot_async(Duration::from_secs(seconds), move |_, _| {
                let bot = bot_clone.clone();
                let text = text_clone.clone();
                let pool = pool_clone.clone();
                let username = username_clone.clone();
                Box::pin(async move {
                    let text = format!(
                        r"From: @{username}

{text}"
                    );
                    if let Err(e) = bot.send_message(chat.id, text).await {
                        tracing::error!("error sending one-off-job {e:#?}");
                    }
                    if let Err(e) = sqlx::query!(
                        r#"UPDATE jobs_one_off 
                         SET 
                         completed = $1
                         WHERE 
                         target = $2
                         and due = $3
                         and username = $4
                         "#,
                        true,
                        chat.id.0,
                        date_time,
                        username
                    )
                    .execute(&pool)
                    .await
                    {
                        tracing::error!("error updating completed one_off_job in database: {e:#?}");
                    }
                })
            })?;

            let job_id = job.guid();
            sqlx::query!(
                r#"INSERT INTO jobs_one_off
            (target, job_id, type, due, completed, message, username)
            VALUES
            ($1, $2, $3, $4, $5, $6, $7)"#,
                chat.id.0,
                job_id,
                "normal",
                date_time,
                false,
                msg_text,
                username
            )
            .execute(&pool)
            .await?;
            sched.add(job).await?;

            p.reset().await?;

            bot.edit_message_text(chat.id, msg_id, "confirmed 🐢 - your message will be sent.")
                .await?;
        }

        _ => expired_callback_msg(bot, chat.id, msg_id).await?,
    }
    Ok(())
}
