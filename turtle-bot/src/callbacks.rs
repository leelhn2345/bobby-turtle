//! Callbacks
//!
//! In each sub-module, there are 2 functions of note.
//!
//! ### page functions
//!
//! these **page** functions determines what is shown in telegram.
//!
//! ### callback functions
//!
//! these **callback** functions decides how the callback data is processed.

mod date;
mod expired;
mod occurrence;
mod remind_text;
mod time;

use ::time::{Date, OffsetDateTime};
pub use date::*;
pub use expired::*;
pub use occurrence::*;
pub use remind_text::*;
use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
pub use time::*;

#[derive(Clone, Default)]

pub enum CallbackPage {
    #[default]
    Expired,
    Occcurence,
    RemindDate,
    RemindDateTime {
        date: Date,
        time: RemindTime,
    },
    ConfirmDateTime {
        date_time: OffsetDateTime,
    },
    ConfirmOneOffJob {
        date_time: OffsetDateTime,
        msg_text: String,
    },
}

pub type CallbackState = Dialogue<CallbackPage, InMemStorage<CallbackPage>>;
