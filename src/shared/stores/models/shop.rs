use chrono::{DateTime, Utc};
use domain::{types::Money, Shop, ShopMember, ShopOffering, UserId};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub(crate) struct ShopRow {
  pub id: Uuid,
  pub owner_user_id: Option<Uuid>,
  pub name: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, FromRow)]
pub(crate) struct ShopOfferingRow {
  pub id: Uuid,
  pub shop_id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub price_cents: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, FromRow)]
pub(crate) struct ShopMemberRow {
  pub id: Uuid,
  pub shop_id: Uuid,
  pub user_id: Uuid,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct ShopCreation {
  pub owner: Option<UserId>,
  pub name: String,
}

#[derive(Clone)]
pub struct ShopUpdate {
  pub owner: Option<Option<UserId>>,
  pub name: Option<String>,
}

#[derive(Clone)]
pub struct ShopOfferingCreation {
  pub name: String,
  pub description: Option<String>,
  pub price: Money,
}

#[derive(Clone)]
pub struct ShopOfferingUpdate {
  pub name: Option<String>,
  pub description: Option<Option<String>>,
  pub price: Option<Money>,
}

impl From<ShopRow> for Shop {
  fn from(value: ShopRow) -> Self {
    Self {
      id: value.id.into(),
      owner: value.owner_user_id.map(Into::into),
      name: value.name,
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}

impl From<ShopOfferingRow> for ShopOffering {
  fn from(value: ShopOfferingRow) -> Self {
    Self {
      id: value.id.into(),
      shop_id: value.shop_id.into(),
      name: value.name,
      description: value.description,
      price_cents: Money::from_minor(value.price_cents),
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}

impl From<ShopMemberRow> for ShopMember {
  fn from(value: ShopMemberRow) -> Self {
    Self {
      id: value.id.into(),
      shop_id: value.shop_id.into(),
      user_id: value.user_id.into(),
      created_at: value.created_at,
      updated_at: value.updated_at,
    }
  }
}
