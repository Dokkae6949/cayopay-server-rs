use crate::models::{
  wallet::{WalletId, WalletLabel},
  Wallet,
};
use crate::stores::models::wallet::{WalletCreation, WalletRow, WalletUpdate};
use sqlx::PgConnection;

pub struct WalletStore;

impl WalletStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &WalletCreation,
  ) -> Result<Wallet, sqlx::Error> {
    let row = sqlx::query_as::<_, WalletRow>(
      r#"
      INSERT INTO wallets (owner_actor_id, label, allow_overdraft)
      VALUES ($1, $2, $3)
      RETURNING id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      "#,
    )
    .bind(creation.owner.map(|o| o.into_inner()))
    .bind(creation.label.as_ref().map(ToString::to_string))
    .bind(creation.allow_overdraft)
    .fetch_one(&mut *conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &WalletId,
    update: &WalletUpdate,
  ) -> Result<Option<Wallet>, sqlx::Error> {
    let row = sqlx::query_as::<_, WalletRow>(
      r#"
      UPDATE wallets
      SET label = CASE WHEN $2 THEN $3 ELSE label END,
          allow_overdraft = COALESCE($4, allow_overdraft)
      WHERE id = $1
      RETURNING id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.label.is_some())
    .bind(
      update
        .label
        .clone()
        .flatten()
        .as_ref()
        .map(ToString::to_string),
    )
    .bind(update.allow_overdraft)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &WalletId,
  ) -> Result<Option<Wallet>, sqlx::Error> {
    let row = sqlx::query_as::<_, WalletRow>(
      r#"
      SELECT id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      FROM wallets
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_label(
    conn: &mut PgConnection,
    label: &WalletLabel,
  ) -> Result<Option<Wallet>, sqlx::Error> {
    let row = sqlx::query_as::<_, WalletRow>(
      r#"
      SELECT id, owner_actor_id, label, allow_overdraft, created_at, updated_at
      FROM wallets
      WHERE label = $1
      "#,
    )
    .bind(label.to_string())
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row.map(Into::into))
  }
}
