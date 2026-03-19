use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use utoipa::openapi::{ObjectBuilder, RefOr, Schema, SchemaFormat, SchemaType};
use utoipa::ToSchema;
use uuid::Uuid;

pub struct Id<T> {
  inner: Uuid,
  phantom: PhantomData<T>,
}

impl<T> Id<T> {
  pub fn new() -> Self {
    Self {
      inner: Uuid::now_v7(),
      phantom: PhantomData,
    }
  }

  pub fn into_inner(self) -> Uuid {
    self.inner
  }

  pub fn cast<U>(self) -> Id<U> {
    Id {
      inner: self.inner,
      phantom: PhantomData,
    }
  }
}

impl<T> Default for Id<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Clone for Id<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
  fn eq(&self, other: &Self) -> bool {
    self.inner == other.inner
  }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.inner.hash(state);
  }
}

impl<T> fmt::Debug for Id<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Id({})", self.inner)
  }
}

impl<T> fmt::Display for Id<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.inner.fmt(f)
  }
}

impl<T> FromStr for Id<T> {
  type Err = uuid::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(Uuid::from_str(s)?))
  }
}

impl<T> From<Uuid> for Id<T> {
  fn from(uuid: Uuid) -> Self {
    Self {
      inner: uuid,
      phantom: PhantomData,
    }
  }
}

impl<T> From<Id<T>> for Uuid {
  fn from(id: Id<T>) -> Self {
    id.inner
  }
}

impl<T> Serialize for Id<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.inner.serialize(serializer)
  }
}

impl<'de, T> Deserialize<'de> for Id<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let uuid = Uuid::deserialize(deserializer)?;
    Ok(Self::from(uuid))
  }
}

impl<T> Type<Postgres> for Id<T> {
  fn type_info() -> PgTypeInfo {
    <Uuid as Type<Postgres>>::type_info()
  }

  fn compatible(ty: &PgTypeInfo) -> bool {
    <Uuid as Type<Postgres>>::compatible(ty)
  }
}

impl<'r, T> Decode<'r, Postgres> for Id<T> {
  fn decode(
    value: PgValueRef<'r>,
  ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let uuid = <Uuid as Decode<Postgres>>::decode(value)?;
    Ok(Self::from(uuid))
  }
}

impl<'q, T> Encode<'q, Postgres> for Id<T> {
  fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> sqlx::encode::IsNull {
    <Uuid as Encode<Postgres>>::encode_by_ref(&self.inner, buf)
  }
}

impl<'s, T> ToSchema<'s> for Id<T> {
  fn schema() -> (&'s str, RefOr<Schema>) {
    (
      "Id",
      ObjectBuilder::new()
        .schema_type(SchemaType::String)
        .format(Some(SchemaFormat::Custom("uuid".to_string())))
        .description(Some("Unique identifier (UUID v7)"))
        .into(),
    )
  }
}
