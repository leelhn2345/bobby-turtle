use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use teloxide::types::Message;

#[derive(thiserror::Error, Debug)]
pub enum ChatRoomError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error("unexpected output from database.")]
    UnexpectedOutput,

    #[error("no record(s) found.")]
    NoRecordFound,

    #[error("chat title did not update.")]
    ChatTitleDidNotUpdate,

    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

pub struct ChatRoom {
    id: i64,
    title: Option<String>,
    joined_at: DateTime<Utc>,
    is_group: bool,
    left_at: Option<DateTime<Utc>>,
}
impl ChatRoom {
    pub fn new(msg: &Message) -> Self {
        let chat_title = msg.chat.title().map(std::borrow::ToOwned::to_owned);

        let is_group = Self::is_group_chat(msg);

        ChatRoom {
            id: msg.chat.id.0,
            title: chat_title,
            joined_at: Utc::now(),
            is_group,
            left_at: None,
        }
    }

    fn is_group_chat(msg: &Message) -> bool {
        !(msg.chat.is_private() || msg.chat.is_channel())
    }

    async fn check_if_exists(
        tx: &mut Transaction<'_, Postgres>,
        chat_id: i64,
    ) -> Result<bool, ChatRoomError> {
        let exists = sqlx::query_scalar!(
            "
            SELECT EXISTS(SELECT 1 FROM chatrooms where id = $1)
            ",
            chat_id
        )
        .fetch_one(&mut **tx)
        .await?;

        if let Some(exists) = exists {
            if exists {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(ChatRoomError::UnexpectedOutput)
        }
    }

    pub async fn leave(pool: &PgPool, chat_id: i64) -> Result<(), ChatRoomError> {
        let mut tx = pool
            .begin()
            .await
            .context("failed to acquire a postgres connection from pool.")?;
        let exists = Self::check_if_exists(&mut tx, chat_id)
            .await
            .context("failed to check if chat id exists in database")?;

        if exists {
            sqlx::query!(
                r#"
                 UPDATE chatrooms
                 SET left_at = $1
                 WHERE id = $2
                 "#,
                Utc::now(),
                chat_id,
            )
            .execute(&mut *tx)
            .await?;
        } else {
            return Err(ChatRoomError::NoRecordFound);
        };
        tx.commit()
            .await
            .context("failed to commit sql transaction to store new chatroom.")?;
        Ok(())
    }

    pub async fn update_title(pool: PgPool, msg: Message) -> anyhow::Result<()> {
        let Some(chat_title) = msg.new_chat_title() else {
            return Err(ChatRoomError::ChatTitleDidNotUpdate.into());
        };

        sqlx::query!(
            "
             UPDATE chatrooms
             SET title = $1
             WHERE id = $2
             ",
            chat_title.to_string(),
            msg.chat.id.0
        )
        .execute(&pool)
        .await?;
        Ok(())
    }

    /// for saving/update new chat room data in database.
    pub async fn save(&self, pool: &PgPool) -> Result<(), ChatRoomError> {
        let mut tx = pool
            .begin()
            .await
            .context("failed to acquire a postgres connection from pool.")?;

        let exists = Self::check_if_exists(&mut tx, self.id)
            .await
            .context("failed to check if chat id exists in database")?;

        if !exists {
            sqlx::query!(
                "
                INSERT INTO chatrooms
                (id, title, is_group, joined_at, left_at)
                VALUES ($1, $2, $3, $4, $5)
            ",
                self.id,
                self.title,
                self.is_group,
                self.joined_at,
                self.left_at
            )
            .execute(&mut *tx)
            .await?;
        } else if self.is_group && exists {
            sqlx::query!(
                "UPDATE chatrooms 
                SET title = $1,
                joined_counter = joined_counter + 1,
                joined_at = $2,
                left_at = $3
                WHERE id = $4",
                self.title,
                self.joined_at,
                self.left_at,
                self.id
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit()
            .await
            .context("failed to commit sql transaction to store new chatroom.")?;

        Ok(())
    }
}
