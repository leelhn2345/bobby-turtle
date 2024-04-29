use teloxide::{types::CallbackQuery, Bot};

#[tracing::instrument(skip_all)]
pub async fn time_pick_callback(bot: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    todo!()
}
