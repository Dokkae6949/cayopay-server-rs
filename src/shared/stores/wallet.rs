use domain::{
  wallet::{WalletId, WalletLabel},
  ActorId, Wallet,
};
use sqlx::{Executor, Postgres};

use crate::stores::models::wallet::{WalletRow, WalletUpdate};

pub struct WalletStore;

impl WalletStore {
  pub async fn create<'c, E>(
    executor: E,
    owner: Option<ActorId>,
    label: Option<WalletLabel>,
    allow_overdraft: bool,
  ) -> Result<Wallet, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      WalletRow,
      r#"
      INSERT INTO wallets (owner_actor_id, label, allow_overdraft)
      VALUES ($1, $2, $3)
      RETURNING id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      "#,
      owner.map(|o| o.into_inner()),
      label.as_ref().map(ToString::to_string),
      allow_overdraft,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &WalletId,
    update: &WalletUpdate,
  ) -> Result<Option<Wallet>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      WalletRow,
      r#"
      UPDATE wallets
      SET label = CASE WHEN $2 THEN $3 ELSE label END,
          allow_overdraft = COALESCE($4, allow_overdraft)
      WHERE id = $1
      RETURNING id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      "#,
      id.into_inner(),
      update.label.is_some(),
      update
        .label
        .clone()
        .flatten()
        .as_ref()
        .map(ToString::to_string),
      update.allow_overdraft,
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &WalletId) -> Result<Option<Wallet>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      WalletRow,
      r#"
      SELECT id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      FROM wallets
      WHERE id = $1
      "#,
      id.into_inner(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_label<'c, E>(
    executor: E,
    label: &WalletLabel,
  ) -> Result<Option<Wallet>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      WalletRow,
      r#"
      SELECT id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      FROM wallets
      WHERE label = $1
      "#,
      label.to_string(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }
}
