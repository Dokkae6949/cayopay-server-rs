use sqlx::{Executor, Postgres};

use crate::shared::stores::models::shop::{
  ShopCreation, ShopMemberRow, ShopOfferingCreation, ShopOfferingRow, ShopOfferingUpdate, ShopRow,
  ShopUpdate,
};
use domain::{Shop, ShopId, ShopMember, ShopMemberId, ShopOffering, ShopOfferingId, UserId};

pub struct ShopStore;

impl ShopStore {
  pub async fn create<'c, E>(executor: E, creation: &ShopCreation) -> Result<Shop, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopRow,
      r#"
      INSERT INTO shops (owner_user_id, name)
      VALUES ($1, $2)
      RETURNING id, owner_user_id, name, created_at, updated_at
      "#,
      creation.owner.map(|id| id.into_inner()),
      creation.name,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &ShopId,
    update: &ShopUpdate,
  ) -> Result<Option<Shop>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopRow,
      r#"
      UPDATE shops
      SET owner_user_id = CASE WHEN $2::boolean THEN $3 ELSE owner_user_id END,
          name = COALESCE($4, name)
      WHERE id = $1
      RETURNING id, owner_user_id, name, created_at, updated_at
      "#,
      id.into_inner(),
      update.owner.is_some(),
      update.owner.flatten().map(|i| i.into_inner()),
      update.name.as_ref(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id<'c, E>(executor: E, id: &ShopId) -> Result<Option<Shop>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopRow,
      r#"
      SELECT id, owner_user_id, name, created_at, updated_at
      FROM shops
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all<'c, E>(executor: E) -> Result<Vec<Shop>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      ShopRow,
      r#"
      SELECT id, owner_user_id, name, created_at, updated_at
      FROM shops
      "#
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}

pub struct ShopOfferingStore;

impl ShopOfferingStore {
  pub async fn create<'c, E>(
    executor: E,
    shop_id: &ShopId,
    creation: &ShopOfferingCreation,
  ) -> Result<ShopOffering, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopOfferingRow,
      r#"
      INSERT INTO shop_offerings (shop_id, name, description, price_cents)
      VALUES ($1, $2, $3, $4)
      RETURNING id, shop_id, name, description, price_cents, created_at, updated_at
      "#,
      shop_id.into_inner(),
      creation.name,
      creation.description.as_ref(),
      creation.price.as_minor() as i32,
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id<'c, E>(
    executor: E,
    id: &ShopOfferingId,
    update: &ShopOfferingUpdate,
  ) -> Result<Option<ShopOffering>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopOfferingRow,
      r#"
      UPDATE shop_offerings
      SET name = COALESCE($2, name),
          description = CASE WHEN $3::boolean THEN $4 ELSE description END,
          price_cents = COALESCE($5, price_cents)
      WHERE id = $1
      RETURNING id, shop_id, name, description, price_cents, created_at, updated_at
      "#,
      id.into_inner(),
      update.name.as_ref(),
      update.description.is_some(),
      update.description.as_ref().and_then(|d| d.as_deref()),
      update.price.map(|p| p.as_minor() as i32),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn delete_by_id<'c, E>(executor: E, id: &ShopOfferingId) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM shop_offerings
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(
    executor: E,
    id: &ShopOfferingId,
  ) -> Result<Option<ShopOffering>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopOfferingRow,
      r#"
      SELECT id, shop_id, name, description, price_cents, created_at, updated_at
      FROM shop_offerings
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_by_shop_id<'c, E>(
    executor: E,
    shop_id: &ShopId,
  ) -> Result<Vec<ShopOffering>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      ShopOfferingRow,
      r#"
      SELECT id, shop_id, name, description, price_cents, created_at, updated_at
      FROM shop_offerings
      WHERE shop_id = $1
      "#,
      shop_id.into_inner()
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}

pub struct ShopMemberStore;

impl ShopMemberStore {
  pub async fn create<'c, E>(
    executor: E,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<ShopMember, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopMemberRow,
      r#"
      INSERT INTO shop_members (shop_id, user_id)
      VALUES ($1, $2)
      RETURNING id, shop_id, user_id, created_at, updated_at
      "#,
      shop_id.into_inner(),
      user_id.into_inner(),
    )
    .fetch_one(executor)
    .await?;

    Ok(row.into())
  }

  pub async fn delete_by_shop_and_user_id<'c, E>(
    executor: E,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<(), sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    sqlx::query!(
      r#"
      DELETE FROM shop_members
      WHERE shop_id = $1 AND user_id = $2
      "#,
      shop_id.into_inner(),
      user_id.into_inner(),
    )
    .execute(executor)
    .await?;

    Ok(())
  }

  pub async fn find_by_id<'c, E>(
    executor: E,
    id: &ShopMemberId,
  ) -> Result<Option<ShopMember>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopMemberRow,
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE id = $1
      "#,
      id.into_inner()
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_shop_and_user_id<'c, E>(
    executor: E,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<Option<ShopMember>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let row = sqlx::query_as!(
      ShopMemberRow,
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE shop_id = $1 AND user_id = $2
      "#,
      shop_id.into_inner(),
      user_id.into_inner(),
    )
    .fetch_optional(executor)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_by_shop_id<'c, E>(
    executor: E,
    shop_id: &ShopId,
  ) -> Result<Vec<ShopMember>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      ShopMemberRow,
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE shop_id = $1
      "#,
      shop_id.into_inner()
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn list_by_user_id<'c, E>(
    executor: E,
    user_id: &UserId,
  ) -> Result<Vec<ShopMember>, sqlx::Error>
  where
    E: Executor<'c, Database = Postgres>,
  {
    let rows = sqlx::query_as!(
      ShopMemberRow,
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE user_id = $1
      "#,
      user_id.into_inner()
    )
    .fetch_all(executor)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
