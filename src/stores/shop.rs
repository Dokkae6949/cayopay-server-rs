use crate::models::{Shop, ShopId, ShopMember, ShopMemberId, ShopOffering, ShopOfferingId, UserId};
use crate::stores::models::shop::{
  ShopCreation, ShopMemberRow, ShopOfferingCreation, ShopOfferingRow, ShopOfferingUpdate, ShopRow,
  ShopUpdate,
};
use sqlx::PgConnection;

pub struct ShopStore;

impl ShopStore {
  pub async fn create(
    conn: &mut PgConnection,
    creation: &ShopCreation,
  ) -> Result<Shop, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopRow>(
      r#"
      INSERT INTO shops (owner_user_id, name)
      VALUES ($1, $2)
      RETURNING id, owner_user_id, name, created_at, updated_at
      "#,
    )
    .bind(creation.owner.map(|id| id.into_inner()))
    .bind(&creation.name)
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &ShopId,
    update: &ShopUpdate,
  ) -> Result<Option<Shop>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopRow>(
      r#"
      UPDATE shops
      SET owner_user_id = CASE WHEN $2::boolean THEN $3 ELSE owner_user_id END,
          name = COALESCE($4, name)
      WHERE id = $1
      RETURNING id, owner_user_id, name, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.owner.is_some())
    .bind(update.owner.flatten().map(|i| i.into_inner()))
    .bind(update.name.as_ref())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &ShopId,
  ) -> Result<Option<Shop>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopRow>(
      r#"
      SELECT id, owner_user_id, name, created_at, updated_at
      FROM shops
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_all(conn: &mut PgConnection) -> Result<Vec<Shop>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShopRow>(
      r#"
      SELECT id, owner_user_id, name, created_at, updated_at
      FROM shops
      "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}

pub struct ShopOfferingStore;

impl ShopOfferingStore {
  pub async fn create(
    conn: &mut PgConnection,
    shop_id: &ShopId,
    creation: &ShopOfferingCreation,
  ) -> Result<ShopOffering, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopOfferingRow>(
      r#"
      INSERT INTO shop_offerings (shop_id, name, description, price_cents)
      VALUES ($1, $2, $3, $4)
      RETURNING id, shop_id, name, description, price_cents, created_at, updated_at
      "#,
    )
    .bind(shop_id.into_inner())
    .bind(&creation.name)
    .bind(creation.description.as_ref())
    .bind(creation.price.as_minor() as i32)
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn update_by_id(
    conn: &mut PgConnection,
    id: &ShopOfferingId,
    update: &ShopOfferingUpdate,
  ) -> Result<Option<ShopOffering>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopOfferingRow>(
      r#"
      UPDATE shop_offerings
      SET name = COALESCE($2, name),
          description = CASE WHEN $3::boolean THEN $4 ELSE description END,
          price_cents = COALESCE($5, price_cents)
      WHERE id = $1
      RETURNING id, shop_id, name, description, price_cents, created_at, updated_at
      "#,
    )
    .bind(id.into_inner())
    .bind(update.name.as_ref())
    .bind(update.description.is_some())
    .bind(update.description.as_ref().and_then(|d| d.as_deref()))
    .bind(update.price.map(|p| p.as_minor() as i32))
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn delete_by_id(
    conn: &mut PgConnection,
    id: &ShopOfferingId,
  ) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shop_offerings WHERE id = $1")
      .bind(id.into_inner())
      .execute(conn)
      .await?;

    Ok(())
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &ShopOfferingId,
  ) -> Result<Option<ShopOffering>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopOfferingRow>(
      r#"
      SELECT id, shop_id, name, description, price_cents, created_at, updated_at
      FROM shop_offerings
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_by_shop_id(
    conn: &mut PgConnection,
    shop_id: &ShopId,
  ) -> Result<Vec<ShopOffering>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShopOfferingRow>(
      r#"
      SELECT id, shop_id, name, description, price_cents, created_at, updated_at
      FROM shop_offerings
      WHERE shop_id = $1
      "#,
    )
    .bind(shop_id.into_inner())
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}

pub struct ShopMemberStore;

impl ShopMemberStore {
  pub async fn create(
    conn: &mut PgConnection,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<ShopMember, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopMemberRow>(
      r#"
      INSERT INTO shop_members (shop_id, user_id)
      VALUES ($1, $2)
      RETURNING id, shop_id, user_id, created_at, updated_at
      "#,
    )
    .bind(shop_id.into_inner())
    .bind(user_id.into_inner())
    .fetch_one(conn)
    .await?;

    Ok(row.into())
  }

  pub async fn delete_by_shop_and_user_id(
    conn: &mut PgConnection,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shop_members WHERE shop_id = $1 AND user_id = $2")
      .bind(shop_id.into_inner())
      .bind(user_id.into_inner())
      .execute(conn)
      .await?;

    Ok(())
  }

  pub async fn find_by_id(
    conn: &mut PgConnection,
    id: &ShopMemberId,
  ) -> Result<Option<ShopMember>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopMemberRow>(
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE id = $1
      "#,
    )
    .bind(id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn find_by_shop_and_user_id(
    conn: &mut PgConnection,
    shop_id: &ShopId,
    user_id: &UserId,
  ) -> Result<Option<ShopMember>, sqlx::Error> {
    let row = sqlx::query_as::<_, ShopMemberRow>(
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE shop_id = $1 AND user_id = $2
      "#,
    )
    .bind(shop_id.into_inner())
    .bind(user_id.into_inner())
    .fetch_optional(conn)
    .await?;

    Ok(row.map(Into::into))
  }

  pub async fn list_by_shop_id(
    conn: &mut PgConnection,
    shop_id: &ShopId,
  ) -> Result<Vec<ShopMember>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShopMemberRow>(
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE shop_id = $1
      "#,
    )
    .bind(shop_id.into_inner())
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }

  pub async fn list_by_user_id(
    conn: &mut PgConnection,
    user_id: &UserId,
  ) -> Result<Vec<ShopMember>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShopMemberRow>(
      r#"
      SELECT id, shop_id, user_id, created_at, updated_at
      FROM shop_members
      WHERE user_id = $1
      "#,
    )
    .bind(user_id.into_inner())
    .fetch_all(conn)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
  }
}
