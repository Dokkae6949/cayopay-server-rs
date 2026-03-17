use chrono::{DateTime, Utc};

use crate::Id;

pub type PermissionId = Id<Permission>;

/// A permission defines an action that can be performed on a subject.
/// e.g. action="send", subject="invite" means the ability to send invites.
#[derive(Debug, Clone)]
pub struct Permission {
  pub id: PermissionId,
  pub action: String,
  pub subject: String,
  pub description: Option<String>,
  pub created_at: DateTime<Utc>,
}
