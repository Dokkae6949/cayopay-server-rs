use crate::domain::Actor;
use crate::error::AppResult;
use crate::{domain::User, domain::Wallet, types::Id};
use sqlx::{Executor, PgConnection, Postgres};

pub struct WalletStore;

impl WalletStore {
  pub async fn save<'c, E>(executor: E, wallet: &Wallet) -> AppResult<()>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      INSERT INTO wallets (id, owner_actor_id, created_at, updated_at)
      VALUES ($1, $2, $3, $4)
      "#,
      wallet.id.into_inner(),
      wallet.owner_actor_id.into_inner(),
      wallet.created_at,
      wallet.updated_at,
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &Id<Wallet>) -> AppResult<Option<Wallet>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let wallet = sqlx::query_as!(
      Wallet,
      r#"
      SELECT id, owner_actor_id, created_at, updated_at
      FROM wallets
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(wallet)
  }

  pub async fn find_by_actor_id<'c, E>(executor: E, actor_id: &Id<Actor>) -> AppResult<Vec<Wallet>>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let wallets = sqlx::query_as!(
      Wallet,
      r#"
      SELECT id, owner_actor_id, created_at, updated_at
      FROM wallets
      WHERE owner_actor_id = $1
      "#,
      actor_id.into_inner()
    )
    .fetch_all(executor)
    .await?;

    Ok(wallets)
  }
}
