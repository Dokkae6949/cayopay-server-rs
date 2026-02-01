use sqlx::{Executor, Postgres};

use crate::{domain::Invite, error::AppResult};

pub struct InviteStore;

impl InviteStore {
  pub async fn create<'c, E>(executor: E, invite: &Invite) -> AppResult<Invite>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let invite = sqlx::query_as!(
      Invite,
      r#"
      INSERT INTO invites (id, created_by, email, token, expires_at, created_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, created_by, email, token, expires_at, created_at
      "#,
      invite.id.into_inner(),
      invite.created_by.into_inner(),
      invite.email.expose(),
      invite.token,
      invite.expires_at,
      invite.created_at
    )
    .fetch_one(executor)
    .await?;

    Ok(invite)
  }
}
