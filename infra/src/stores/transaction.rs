use domain::{transaction::TransactionId, types::Money, wallet::WalletId, Transaction};
use sqlx::{Executor, Postgres};

use crate::stores::models::transaction::{TransactionCreation, TransactionRow};

pub struct TransactionStore;

impl TransactionStore {
  pub async fn create<'c, E>(
    executor: E,
    creation: &TransactionCreation,
  ) -> Result<Transaction, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      TransactionRow,
      r#"
      INSERT INTO transactions (source_wallet_id, destination_wallet_id, executor_actor_id, amount, description)
      VALUES ($1, $2, $3, $4, $5)
      RETURNING id, source_wallet_id, destination_wallet_id, executor_actor_id, amount, description, created_at, updated_at
      "#,
      creation.source.into_inner(),
      creation.destination.into_inner(),
      creation.executor.as_ref().map(|e| e.into_inner()),
      creation.amount.as_minor() as i64,
      creation.description,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn find_by_id<'c, E>(
    executor: E,
    id: &TransactionId,
  ) -> Result<Option<Transaction>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      TransactionRow,
      r#"
      SELECT id, source_wallet_id, destination_wallet_id, executor_actor_id, amount, description, created_at, updated_at
      FROM transactions
      WHERE id = $1
      "#,
      id.into_inner(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_wallet_id<'c, E>(
    executor: E,
    wallet_id: &WalletId,
  ) -> Result<Vec<Transaction>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      TransactionRow,
      r#"
      SELECT id, source_wallet_id, destination_wallet_id, executor_actor_id, amount, description, created_at, updated_at
      FROM transactions
      WHERE source_wallet_id = $1 OR destination_wallet_id = $1
      ORDER BY created_at DESC
      "#,
      wallet_id.into_inner(),
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn calculate_wallet_balance<'c, E>(
    executor: E,
    wallet_id: &WalletId,
  ) -> Result<Money, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let balance = sqlx::query_scalar!(
      r#"
      SELECT
        COALESCE(SUM(
          CASE
            WHEN destination_wallet_id = $1 THEN amount
            WHEN source_wallet_id = $1 THEN -amount
            ELSE 0
          END
        ), 0) AS balance
      FROM transactions
      WHERE source_wallet_id = $1 OR destination_wallet_id = $1
      "#,
      wallet_id.into_inner(),
    )
    .fetch_one(executor)
    .await?;

    Ok(Money::from_minor(balance.unwrap_or_default().max(0) as u64))
  }
}
