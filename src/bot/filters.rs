use std::sync::OnceLock;

use teloxide::types::Message;

use crate::bot::BOT_ME;

#[allow(clippy::needless_pass_by_value)]
pub fn is_group_chat(msg: Message) -> bool {
    if msg.chat.is_private() || msg.chat.is_channel() {
        return false;
    }
    true
}

pub fn is_not_group_chat(msg: Message) -> bool {
    !is_group_chat(msg)
}

#[allow(clippy::needless_pass_by_value)]
pub fn group_title_change(msg: Message) -> bool {
    msg.new_chat_title().is_some()
}

#[tracing::instrument(skip_all)]
#[allow(clippy::needless_pass_by_value)]
/// Checks if the bot's name is mentioned in the message.
///
/// if `true`, user is chatting with bot.
/// returns `false` by default.
pub fn to_bot(msg: Message) -> bool {
    static BOT_NAME: OnceLock<String> = OnceLock::new();

    let text = match msg.text() {
        None => return false,
        Some(x) => {
            if x.is_empty() {
                return false;
            }
            x
        }
    };

    let name = BOT_NAME.get_or_init(|| {
        let first_name: Vec<&str> = BOT_ME
            .get()
            .unwrap()
            .first_name
            .split_whitespace()
            .collect();
        first_name.first().copied().unwrap().to_lowercase()
    });

    tracing::debug!(text);
    tracing::debug!(name);

    text.contains(name)
}
