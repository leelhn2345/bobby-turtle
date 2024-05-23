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

use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;

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
        date: NaiveDate,
        time: RemindTime,
    },
    ConfirmDateTime {
        date_time: DateTime<Tz>,
    },
    ConfirmOneOffJob {
        date_time: DateTime<Tz>,
        msg_text: String,
    },
}

pub type CallbackState = Dialogue<CallbackPage, InMemStorage<CallbackPage>>;
