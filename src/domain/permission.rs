use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  ConfigureSettings,

  RemoveActor,
  ReadActorDetails,

  InviteUser,
  RemoveUser,
  ReadUserDetails,

  RemoveGuest,
  ReadGuestDetails,
}
