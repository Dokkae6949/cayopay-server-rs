use crate::error::AppResult;
use crate::{domain::Transaction, domain::Wallet, types::Id};
use sqlx::{Executor, PgConnection, Postgres};

pub struct TransactionStore;

impl TransactionStore {
  pub async fn save<'c, E>(executor: E, transaction: &Transaction) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      INSERT INTO transactions (id, sender_wallet_id, receiver_wallet_id, executor_actor_id, amount, description, created_at, updated_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      "#,
      transaction.id.into_inner(),
      transaction.sender_wallet_id.into_inner(),
      transaction.receiver_wallet_id.into_inner(),
      transaction.executor_actor_id.into_inner(),
      transaction.amount,
      transaction.description,
      transaction.created_at,
      transaction.updated_at,
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(
    executor: E,
    id: &Id<Transaction>,
  ) -> AppResult<Option<Transaction>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let transaction = sqlx::query_as!(
      Transaction,
      r#"
      SELECT id, sender_wallet_id, receiver_wallet_id, executor_actor_id, amount, description, created_at, updated_at
      FROM transactions
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(transaction)
  }

  pub async fn balance_by_wallet_id<'c, E>(executor: E, wallet_id: &Id<Wallet>) -> AppResult<i64>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let balance: Option<i64> = sqlx::query_scalar!(
      r#"
      SELECT COALESCE(SUM(
        CASE
          WHEN receiver_wallet_id = $1 THEN amount
          WHEN sender_wallet_id = $1 THEN -amount
          ELSE 0
        END
      ), 0) AS balance
      FROM transactions
      "#,
      wallet_id.into_inner()
    )
    .fetch_one(executor)
    .await?;

    Ok(balance.unwrap_or(0))
  }
}

#[cfg(test)]
mod tests {
  use sqlx::PgPool;

  use crate::{
    domain::Actor,
    stores::{ActorStore, WalletStore},
  };

  use super::*;

  #[sqlx::test]
  async fn test_balance_calculation(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let red_actor = Actor::new();
    let blue_actor = Actor::new();

    let red_wallet = Wallet::new(red_actor.id);
    let blue_wallet = Wallet::new(blue_actor.id);

    ActorStore::save(&pool, &red_actor).await?;
    ActorStore::save(&pool, &blue_actor).await?;

    WalletStore::save(&pool, &red_wallet).await?;
    WalletStore::save(&pool, &blue_wallet).await?;

    let transactions = vec![
      Transaction {
        id: Id::new(),
        sender_wallet_id: red_wallet.id,
        receiver_wallet_id: blue_wallet.id,
        executor_actor_id: red_actor.id,
        amount: 100,
        description: Some("Payment 1".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      },
      Transaction {
        id: Id::new(),
        sender_wallet_id: blue_wallet.id,
        receiver_wallet_id: red_wallet.id,
        executor_actor_id: blue_actor.id,
        amount: 50,
        description: Some("Payment 2".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      },
      Transaction {
        id: Id::new(),
        sender_wallet_id: red_wallet.id,
        receiver_wallet_id: blue_wallet.id,
        executor_actor_id: red_actor.id,
        amount: 30,
        description: Some("Payment 3".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      },
    ];

    for tx in &transactions {
      TransactionStore::save(&pool, tx).await?;
    }

    let red_balance = TransactionStore::balance_by_wallet_id(&pool, &red_wallet.id).await?;
    let blue_balance = TransactionStore::balance_by_wallet_id(&pool, &blue_wallet.id).await?;

    assert_eq!(red_balance, -80); //  -100 + 50 - 30
    assert_eq!(blue_balance, 80); //  +100 - 50 + 30

    Ok(())
  }
}
