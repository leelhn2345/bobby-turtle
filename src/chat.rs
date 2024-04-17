use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use teloxide::types::Message;

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

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(128_u16)
        .model("gpt-3.5-turbo")
        .messages([sys_msg, chat_req])
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
