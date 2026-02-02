use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::{Actor, Wallet};
use crate::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Transaction {
  pub id: Id<Transaction>,
  pub sender_wallet_id: Id<Wallet>,
  pub receiver_wallet_id: Id<Wallet>,
  pub executor_actor_id: Id<Actor>,
  /// Amount in minor currency units (e.g. cents)
  pub amount: i32,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Transaction {
  pub fn new(
    sender_wallet_id: Id<Wallet>,
    receiver_wallet_id: Id<Wallet>,
    executor_actor_id: Id<Actor>,
    amount: i32,
    description: Option<String>,
  ) -> Self {
    let now = Utc::now();
    Self {
      id: Id::new(),
      sender_wallet_id,
      receiver_wallet_id,
      executor_actor_id,
      amount,
      description,
      created_at: now,
      updated_at: now,
    }
  }
}
