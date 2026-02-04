use chrono::{DateTime, Utc};

use crate::{types::Money, Id, UserId};

pub type ShopId = Id<Shop>;
pub type ShopOfferingId = Id<ShopOffering>;
pub type ShopMemberId = Id<ShopMember>;

#[derive(Debug, Clone)]
pub struct Shop {
  pub id: ShopId,
  pub owner: Option<UserId>,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ShopOffering {
  pub id: ShopOfferingId,
  pub shop_id: ShopId,
  pub name: String,
  pub description: Option<String>,
  pub price_cents: Money,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ShopMember {
  pub id: ShopMemberId,
  pub shop_id: ShopId,
  pub user_id: UserId,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
