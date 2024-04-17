mod commands;
mod member;

use anyhow::Result;
use teloxide::{
    dispatching::{DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt},
    dptree::{self, di::DependencyMap, Handler},
    types::{Message, Update},
};

pub fn bot_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<commands::Command>()
                        .endpoint(commands::answer),
                )
                .branch(Message::filter_new_chat_members().endpoint(member::handle_member_join))
                .branch(Message::filter_left_chat_member().endpoint(member::handle_member_leave)),
        )
}
