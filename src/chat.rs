use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, Role,
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
    EmptyMessageFromUser(&'static str),

    #[error(transparent)]
    RequestError(#[from] RequestError),

    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    UnknownError(#[from] serde_json::Error),
}

#[tracing::instrument(skip_all)]
pub async fn user_chat(
    bot: Bot,
    client: Client<OpenAIConfig>,
    msg: Message,
    pool: PgPool,
) -> anyhow::Result<()> {
    if let Some(x) = msg.text() {
        tracing::debug!("some1 is chatting with bot");
        let chat_msg = x.to_string();
        bot_chat(bot, client, &msg, chat_msg, pool).await?;
    }
    Ok(())
}

#[tracing::instrument(name = "bot chatting", skip_all)]
#[allow(deprecated)]
pub async fn bot_chat(
    bot: Bot,
    client: Client<OpenAIConfig>,
    msg: &Message,
    chat_msg: String,
    pool: PgPool,
) -> Result<Message, ChatError> {
    let chat_response = match chatgpt_chat(client, msg, chat_msg, pool).await {
        Ok(response) => {
            bot.send_message(msg.chat.id, response)
                .parse_mode(ParseMode::Markdown)
                .reply_to_message_id(msg.id)
                .await?
        }
        Err(e) => {
            if let ChatError::EmptyMessageFromUser(empty_msg_response) = e {
                bot.send_message(msg.chat.id, empty_msg_response)
                    .parse_mode(ParseMode::Markdown)
                    .reply_to_message_id(msg.id)
                    .await?
            } else {
                tracing::error!("{:#?}", e);
                return Err(e);
            }
        }
    };
    Ok(chat_response)
}

#[tracing::instrument(name = "chatgpt's chat completion", skip_all)]
pub async fn chatgpt_chat(
    client: Client<OpenAIConfig>,
    msg: &Message,
    chat_msg: String,
    pool: PgPool,
) -> Result<String, ChatError> {
    if chat_msg.is_empty() {
        return Err(ChatError::EmptyMessageFromUser(
            "Hello! Feel free to chat with me! 😊
            \nExample: `/chat how are you?`",
        ));
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

    save_logs(&mut tx, msg.chat.id.0, Role::User, &chat_msg, username).await?;

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
            "You are a cute bubbly turtle and your name is {}.",
            BOT_NAME.get().unwrap()
        ))
        .build()?
        .into();

    let mut chat_cmp_msg = vec![sys_msg];
    chat_cmp_msg.push(chat_req);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(128_u16)
        .model("gpt-3.5-turbo")
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

    save_logs(
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
async fn get_logs(
    _tx: Transaction<'_, Postgres>,
    _msg_id: i64,
) -> Result<Vec<ChatCompletionRequestMessage>, ChatError> {
    todo!()
}

#[tracing::instrument(name = "save chat logs to db", skip_all)]
async fn save_logs(
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
        tracing::error!("{e:#?}");
        e
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    enum Abc {
        A(A),
        B(B),
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct A {
        a: u8,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct B {
        b: C,
    }
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    enum C {
        D(String),
    }
    #[test]
    fn vec_parse() {
        let aaa: Vec<Abc> = vec![
            Abc::A(A { a: 9 }),
            Abc::B(B {
                b: C::D("fefefe".to_string()),
            }),
        ];
        let fff = serde_json::to_string_pretty(&aaa).expect("cannot serialize");
        println!("{fff:#?}");
        let eee: Vec<Abc> = serde_json::from_str(&fff).expect("cannot deserialize");
        println!("{eee:#?}");
    }
}
