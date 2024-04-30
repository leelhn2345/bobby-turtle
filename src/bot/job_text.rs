use std::time::Duration;

use anyhow::bail;
use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    Bot,
};
use tokio_cron_scheduler::{Job, JobScheduler};

use super::{expired_callback_msg, time_pick::CHANGE_TIME, CallbackDialogue, CallbackState};

pub const JOB_TEXT_BACK: &str = "Back";
pub const JOB_TEXT_CONFIRM: &str = "Confirm";

fn job_text_keyboard() -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![vec![
        InlineKeyboardButton::callback(JOB_TEXT_BACK, JOB_TEXT_BACK),
        InlineKeyboardButton::callback(JOB_TEXT_CONFIRM, JOB_TEXT_CONFIRM),
    ]];

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn register_job_text(
    bot: Bot,
    msg: Message,
    chosen_datetime: DateTime<Tz>,
    callback: CallbackDialogue,
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
        .update(CallbackState::ConfirmOneOffJob {
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

#[tracing::instrument(skip_all)]
pub async fn one_off_job_callback(
    bot: Bot,
    q: CallbackQuery,
    callback: CallbackDialogue,
    (date_time, msg_text): (DateTime<Tz>, String),
    pool: PgPool,
    sched: JobScheduler,
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id).await?;

    let Some(data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        bail!("no query callback data")
    };
    let Some(Message { id, chat, .. }) = q.message else {
        tracing::error!("no message data from telegram");
        bail!("no telegram message data")
    };

    let now = Utc::now().with_timezone(&Tz::Singapore);
    if date_time < now {
        tracing::error!("chosen datetime is in the past");
        let current_time = now.time().format("%H:%M:%S").to_string();
        let text = format!(
            r"You can't send a message into the past. ❌

Messages should be after this instant.
The current time is {current_time}."
        );
        bot.send_message(chat.id, text).await?;

        bail!("chosen datetime can't be before this current instant");
    }

    match data.as_ref() {
        JOB_TEXT_BACK => {
            callback
                .update(CallbackState::ConfirmDateTime { date_time })
                .await?;
            let chosen_year = date_time.year();
            let chosen_month = date_time.month();
            let chosen_day = date_time.day();
            let chosen_hour = date_time.hour();
            let chosen_minute = date_time.minute();

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

            bot.edit_message_text(chat.id, id, text)
                .reply_markup(InlineKeyboardMarkup::new(vec![vec![
                    InlineKeyboardButton::callback("Back", CHANGE_TIME),
                ]]))
                .await?;
        }
        JOB_TEXT_CONFIRM => {
            let time_delta = date_time - Utc::now().with_timezone(&Tz::Singapore);
            let time_delta_secs = time_delta.num_seconds();
            let seconds = u64::from_ne_bytes(time_delta_secs.to_ne_bytes());

            let bot_clone = bot.clone();
            let text_clone = msg_text.clone();
            let job = Job::new_one_shot_async(Duration::from_secs(seconds), move |_, _| {
                let bot = bot_clone.clone();
                let text = text_clone.clone();
                Box::pin(async move {
                    if let Err(e) = bot.send_message(chat.id, text).await {
                        tracing::error!("error sending one-off-job {e:#?}");
                    }
                })
            })?;
            sched.add(job).await?;

            bot.edit_message_text(
                chat.id,
                id,
                format!("confirmed 🐢 - your text is {msg_text}"),
            )
            .await?;
            callback.reset().await?;
        }
        _ => expired_callback_msg(bot, chat, id).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use chrono_tz::Tz;

    #[test]
    fn zzzzz() {
        let wow = Utc::now().with_timezone(&Tz::Singapore).to_string();
        println!("{wow}");

        let hmm = DateTime::parse_from_rfc3339(&wow).unwrap();
        println!("{hmm}");
    }
}
