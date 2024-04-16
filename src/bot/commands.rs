use teloxide::{
    requests::Requester,
    types::{ChatId, InputFile, Message},
    utils::command::BotCommands,
    Bot,
};

use crate::settings::stickers::Stickers;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command()]
    Help,
    #[command(description = "chat with me!")]
    Chat(String),
    #[command(description = "current datetime.")]
    DateTime,
    #[command(description = "feed me.")]
    Feed,
}

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    stickers: Stickers,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;
    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string())
                .await?
        }
        Command::DateTime => {
            send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
            bot.send_message(chat_id, "~ feature coming soon ~").await?
        }
        Command::Chat(qns) => {
            tracing::debug!(qns);
            send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
            bot.send_message(chat_id, "~ feature coming soon ~").await?
        }
        Command::Feed => {
            send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
            bot.send_message(chat_id, "~ feature coming soon ~").await?
        }
    };
    Ok(())
}

pub async fn send_sticker(bot: &Bot, chat_id: &ChatId, sticker_id: String) -> anyhow::Result<()> {
    bot.send_sticker(*chat_id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_openai::{
        types::{
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs, CreateCompletionRequestArgs,
        },
        Client,
    };
    use dotenvy::dotenv;

    #[tokio::test]
    async fn chatgpt() {
        dotenv().unwrap();
        // Create client
        let client = Client::new();

        // Create request using builder pattern
        // Every request struct has companion builder struct with same name + Args suffix
        let request = CreateCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .prompt("who is the best basketball player of all time?")
            .max_tokens(128_u16)
            .build()
            .unwrap();

        // Call API
        let response = client
            .completions() // Get the API "group" (completions, images, etc.) from the client
            .create(request) // Make the API call in that "group"
            .await
            .unwrap();

        println!("{}", response.choices.first().unwrap().text);
    }

    #[tokio::test]
    async fn chat_gpt() {
        dotenv().unwrap();
        let client = Client::new();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(128_u16)
            .model("gpt-3.5-turbo")
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("you are a helpful assistant that ends every sentence with a meow.")
                    .build()
                    .unwrap()
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content("Who won the world series in 2020?")
                    .name("bobby")
                    .build()
                    .unwrap()
                    .into(),
                // ChatCompletionRequestAssistantMessageArgs::default()
                //     .content("The Los Angeles Dodgers won the World Series in 2020.")
                //     .build()
                //     .unwrap()
                //     .into(),
                // ChatCompletionRequestUserMessageArgs::default()
                //     .content("Where was it played?")
                //     .build()
                //     .unwrap()
                //     .into(),
            ])
            .build()
            .unwrap();

        println!("{:#?}", serde_json::to_string(&request).unwrap());
        println!("{:#?}", &request);

        let response = client.chat().create(request).await.unwrap();

        println!("\nResponse:\n");
        println!("{:#?}", response.choices.first().unwrap().message.content);
        for choice in response.choices {
            println!(
                "{}: Role: {}  Content: {:?}",
                choice.index, choice.message.role, choice.message.content
            );
        }
    }
}
