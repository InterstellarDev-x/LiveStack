use crate::Store;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::channel_link)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelLink {
    pub id: String,
    pub channel: String,
    pub channel_user_id: String,
    pub user_id: Option<String>,
    pub pairing_code: String,
    /// JSON-serialized `Vec<ai::ChatMessage>`; store stays agnostic of the
    /// `ai` crate's types, so callers own (de)serialization.
    pub history: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

fn new_pairing_code() -> String {
    Uuid::new_v4().simple().to_string()[..6].to_uppercase()
}

/// Pairing codes are 6 characters, so collisions are rare but not negligible.
/// The unique index turns one into a failed insert, which is retryable —
/// nobody involved did anything wrong.
const PAIRING_CODE_ATTEMPTS: usize = 5;

impl Store {
    /// Worker-facing, no owner check: called by the channel gateway before a
    /// sender is known to be linked to anyone. Returns the existing row if
    /// this (channel, channel_user_id) pair has already messaged before,
    /// otherwise creates a fresh pending link with a new pairing code.
    pub fn find_or_create_channel_link(
        &mut self,
        input_channel: &str,
        input_channel_user_id: &str,
    ) -> Result<ChannelLink, diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        let mut last_conflict = None;

        for _ in 0..PAIRING_CODE_ATTEMPTS {
            if let Some(existing) = channel_link
                .filter(channel.eq(input_channel))
                .filter(channel_user_id.eq(input_channel_user_id))
                .select(ChannelLink::as_select())
                .first(self.conn())
                .optional()?
            {
                return Ok(existing);
            }

            let now = Utc::now().naive_utc();
            let new_link = ChannelLink {
                id: Uuid::new_v4().to_string(),
                channel: input_channel.to_string(),
                channel_user_id: input_channel_user_id.to_string(),
                user_id: None,
                pairing_code: new_pairing_code(),
                history: "[]".to_string(),
                created_at: now,
                updated_at: now,
            };

            match diesel::insert_into(channel_link)
                .values(&new_link)
                .returning(ChannelLink::as_returning())
                .get_result(self.conn())
            {
                Ok(link) => return Ok(link),
                // Either this chat raced another message from itself (the
                // loop's re-select above picks up the winner) or the generated
                // code collided with an existing one (a fresh code fixes it).
                Err(err @ diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => last_conflict = Some(err),
                Err(err) => return Err(err),
            }
        }

        // Every attempt collided, which realistically means something else is
        // wrong; surface the database's own error rather than inventing one.
        Err(last_conflict.unwrap_or(diesel::result::Error::NotFound))
    }

    /// Owner-facing: called with the JWT-authenticated user_id from the
    /// linking UI. 404s (as `NotFound`) if the code doesn't match a pending
    /// link, so a guessed/expired code can't be used to probe for one.
    pub fn approve_channel_link(
        &mut self,
        input_pairing_code: &str,
        input_user_id: &str,
    ) -> Result<ChannelLink, diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        diesel::update(
            channel_link
                .filter(pairing_code.eq(input_pairing_code))
                .filter(user_id.is_null()),
        )
        .set((
            user_id.eq(Some(input_user_id.to_string())),
            updated_at.eq(Utc::now().naive_utc()),
        ))
        .returning(ChannelLink::as_returning())
        .get_result(self.conn())
        .optional()?
        .ok_or(diesel::result::Error::NotFound)
    }

    pub fn list_channel_links_for_user(
        &mut self,
        input_user_id: &str,
    ) -> Result<Vec<ChannelLink>, diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        channel_link
            .filter(user_id.eq(input_user_id))
            .select(ChannelLink::as_select())
            .load(self.conn())
    }

    /// Owner-checked delete. Returns `NotFound` if the link doesn't exist or
    /// isn't owned by `input_user_id`.
    pub fn unlink_channel_link(
        &mut self,
        input_id: &str,
        input_user_id: &str,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        let deleted = diesel::delete(
            channel_link
                .filter(id.eq(input_id))
                .filter(user_id.eq(input_user_id)),
        )
        .execute(self.conn())?;

        if deleted == 0 {
            return Err(diesel::result::Error::NotFound);
        }
        Ok(())
    }

    /// Worker-facing: looks up a link by channel identity regardless of
    /// linked status, so the gateway can decide what to do next.
    pub fn get_channel_link(
        &mut self,
        input_channel: &str,
        input_channel_user_id: &str,
    ) -> Result<Option<ChannelLink>, diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        channel_link
            .filter(channel.eq(input_channel))
            .filter(channel_user_id.eq(input_channel_user_id))
            .select(ChannelLink::as_select())
            .first(self.conn())
            .optional()
    }

    pub fn save_channel_link_history(
        &mut self,
        input_id: &str,
        input_history: &str,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::channel_link::dsl::*;

        diesel::update(channel_link.filter(id.eq(input_id)))
            .set((
                history.eq(input_history),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(self.conn())?;
        Ok(())
    }
}
