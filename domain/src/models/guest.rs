use chrono::{DateTime, Utc};

use crate::{actor::ActorId, Email, Id};

pub type GuestId = Id<Guest>;

#[derive(Debug, Clone)]
pub struct Guest {
  pub id: GuestId,
  pub actor_id: ActorId,
  pub email: Option<Email>,
  pub verified: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
