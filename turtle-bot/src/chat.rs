use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode},
    Bot, RequestError,
};

use crate::bot::BOT_NAME;

/// number of tokens from chatgpt's response
const MAX_TOKENS: u16 = 512;
/// chatgpt model used for query
const MODEL: &str = "gpt-3.5-turbo";
// number of past chat records to retrieve
const PAST_LOG_COUNT: i64 = 20;

#[derive(thiserror::Error, Debug)]
pub enum ChatError {
    #[error(transparent)]
    OpenAIError(#[from] OpenAIError),

    #[error("no chat completion choices available.")]
    NoChatCompletion,

    #[error("bot did not respond")]
    NoContent,

    #[error("chat was prompted by bot.")]
    IsBot,

    #[error("no chat message provided by user.")]
    EmptyMessageFromUser,

    #[error(transparent)]
    RequestError(#[from] RequestError),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

#[tracing::instrument(skip_all)]
pub async fn user_chat(
    bot: Bot,
    client: Client<OpenAIConfig>,
    msg: Message,
    pool: PgPool,
) -> anyhow::Result<()> {
    if let Some(chat_msg) = msg.text() {
        tracing::debug!("some1 is chatting with bot");
        bot_chat(bot, client, &msg, chat_msg, pool).await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
#[allow(deprecated)]
pub async fn bot_chat(
    bot: Bot,
    client: Client<OpenAIConfig>,
    msg: &Message,
    chat_msg: impl Into<String>,
    pool: PgPool,
) -> Result<Message, ChatError> {
    let chat_response = match chatgpt_chat(client, msg, chat_msg.into(), pool).await {
        Ok(response) => {
            bot.send_message(msg.chat.id, response)
                .parse_mode(ParseMode::Markdown)
                .reply_to_message_id(msg.id)
                .await?
        }
        Err(e) => {
            if let Some(username) = msg.chat.username() {
                tracing::error!("{} chatted with bot.\n{e:#?}", username);
            } else {
                tracing::error!("{e:#?}");
            }
            return Err(e);
        }
    };
    Ok(chat_response)
}

#[tracing::instrument(skip_all)]
pub async fn chatgpt_chat(
    client: Client<OpenAIConfig>,
    msg: &Message,
    chat_msg: String,
    pool: PgPool,
) -> Result<String, ChatError> {
    if chat_msg.is_empty() {
        return Err(ChatError::EmptyMessageFromUser);
    }
    if let Some(user) = &msg.via_bot {
        if user.is_bot {
            return Err(ChatError::IsBot);
        }
    };

    let mut tx = pool.begin().await?;

    let username = match msg.from() {
        Some(user) => match &user.username {
            Some(username) => Some(username),
            None => None,
        },
        None => None,
    };
    let mut past_logs = get_logs(&mut tx, msg.chat.id.0).await?;

    save_chat_logs(&mut tx, msg.chat.id.0, Role::User, &chat_msg, username).await?;

    let chat_req = match username {
        Some(x) => ChatCompletionRequestUserMessageArgs::default()
            .content(chat_msg)
            .name(x)
            .build()?
            .into(),

        None => ChatCompletionRequestUserMessageArgs::default()
            .content(chat_msg)
            .build()?
            .into(),
    };

    let sys_msg = ChatCompletionRequestSystemMessageArgs::default()
        .content(format!(
            "You are a cute, bubbly, 190 years old, heterogenous male turtle and your name is {}.",
            BOT_NAME.get().unwrap()
        ))
        .build()?
        .into();

    let mut chat_cmp_msg = vec![sys_msg];
    chat_cmp_msg.append(&mut past_logs);
    chat_cmp_msg.push(chat_req);
    tracing::debug!("chat_cmp_msg is {chat_cmp_msg:#?}");

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(MAX_TOKENS)
        .model(MODEL)
        .messages(chat_cmp_msg)
        .build()?;

    let response = client.chat().create(request).await?;

    let chat_response = response
        .choices
        .first()
        .ok_or(ChatError::NoChatCompletion)?
        .message
        .content
        .as_ref()
        .ok_or(ChatError::NoContent)?
        .to_owned();

    save_chat_logs(
        &mut tx,
        msg.chat.id.0,
        Role::Assistant,
        &chat_response,
        None,
    )
    .await?;

    tx.commit().await?;

    Ok(chat_response)
}
struct PastMsg {
    name: Option<String>,
    content: String,
    role: String,
}

/// The role is saved as string in database.
/// I am matching the string to it's particular role.
/// Approach is quite dangerous but i can't think of a safer way.
/// It is better to match it programmatically than matching it to `&str` type.
///
/// If it can't parse any role, it won't be able to get previous chat messages.
/// Function will just return an empty vector.
/// The silver lining is that less tokens will be sent to OpenAI,
/// resulting in lower costs.
#[tracing::instrument(skip_all)]
async fn get_logs(
    tx: &mut Transaction<'_, Postgres>,
    msg_id: i64,
) -> Result<Vec<ChatCompletionRequestMessage>, ChatError> {
    let past_msges: Vec<PastMsg> = sqlx::query_as!(
        PastMsg,
        r#"
        SELECT name, role, content FROM chatlogs
        WHERE message_id = $1
        AND datetime>= CURRENT_TIMESTAMP - INTERVAL '1 hour'
        ORDER BY datetime DESC
        LIMIT $2
        "#,
        msg_id,
        PAST_LOG_COUNT
    )
    .fetch_all(&mut **tx)
    .await?;
    let mut past_req_msges: Vec<ChatCompletionRequestMessage> = past_msges
        .into_iter()
        .map(|x| match x.role.to_lowercase().as_str().trim() {
            "user" => Ok(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .name(x.name.unwrap_or_default())
                    .content(x.content)
                    .build()?,
            )),
            "assistant" => Ok(ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(x.content)
                    .build()?,
            )),
            _ => Err(OpenAIError::InvalidArgument("invalid role".to_string())),
        })
        .filter_map(std::result::Result::ok)
        .collect();
    past_req_msges.reverse();
    tracing::debug!("{past_req_msges:#?}");
    Ok(past_req_msges)
}

#[tracing::instrument(skip_all)]
async fn save_chat_logs(
    tx: &mut Transaction<'_, Postgres>,
    msg_id: i64,
    role: Role,
    content: &String,
    username: Option<&String>,
) -> Result<(), ChatError> {
    let role_str = role.to_string();
    sqlx::query!(
        r#"
        INSERT INTO chatlogs 
        (message_id, name, role, content, datetime)
        VALUES ($1, $2, $3, $4, $5) 
        "#,
        msg_id,
        username,
        role_str,
        content,
        Utc::now()
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e);
        e
    })?;
    Ok(())
}
