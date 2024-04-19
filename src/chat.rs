use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode},
    Bot, RequestError,
};

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
}

#[tracing::instrument(name = "user chatting with bot", skip_all)]
pub async fn user_chat(bot: Bot, client: Client<OpenAIConfig>, msg: Message) -> anyhow::Result<()> {
    if let Some(x) = msg.text() {
        let chat_msg = x.to_string();
        bot_chat(bot, client, &msg, chat_msg).await?;
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
) -> Result<Message, ChatError> {
    let chat_response = match chatgpt_chat(client, msg, chat_msg).await {
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

    let username_exists = match msg.from() {
        Some(user) => match &user.username {
            Some(username) => Some(username),
            None => None,
        },
        None => None,
    };

    let chat_req = match username_exists {
        Some(username) => ChatCompletionRequestUserMessageArgs::default()
            .content(chat_msg)
            .name(username)
            .build()?
            .into(),

        None => ChatCompletionRequestUserMessageArgs::default()
            .content(chat_msg)
            .build()?
            .into(),
    };

    let sys_msg = ChatCompletionRequestSystemMessageArgs::default()
        .content("You are a cute bubbly turtle.")
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

    Ok(chat_response)
}
