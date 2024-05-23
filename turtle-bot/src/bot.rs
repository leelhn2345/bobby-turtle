use anyhow::Result;
use teloxide::{
    dispatching::{DpHandlerDescription, UpdateFilterExt},
    dptree::{self, di::DependencyMap, Handler},
    types::Update,
};

pub fn bot_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    todo!()
}
