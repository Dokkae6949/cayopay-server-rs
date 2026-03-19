/// All permissions that can be granted to a role are defined here in code.
///
/// Keeping permissions as an enum provides compile-time safety: an endpoint that
/// wants to gate on `Permission::SendInvite` cannot accidentally misspell the
/// string at the call-site.  Only *roles* are dynamic (stored in the database).
///
/// The string representation (see [`Permission::as_str`]) is what gets stored in
/// the `role_permissions.permission` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
  /// Configure system-wide settings.
  ConfigureSettings,
  /// Send an invite to a new user.
  SendInvite,
  /// View all existing invites.
  ViewInvite,
  /// Remove a user from the system.
  RemoveUser,
  /// Read user details.
  ReadUser,
  /// Remove a guest from the system.
  RemoveGuest,
  /// Read guest details.
  ReadGuest,
}

impl Permission {
  /// The canonical string code that is stored in the database.
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::ConfigureSettings => "settings.configure",
      Permission::SendInvite => "invite.send",
      Permission::ViewInvite => "invite.view",
      Permission::RemoveUser => "user.remove",
      Permission::ReadUser => "user.read",
      Permission::RemoveGuest => "guest.remove",
      Permission::ReadGuest => "guest.read",
    }
  }

  /// Attempts to parse a permission from its string code.
  /// Returns `None` for unknown codes (e.g. stale DB entries after a code update).
  pub fn from_code(s: &str) -> Option<Self> {
    match s {
      "settings.configure" => Some(Permission::ConfigureSettings),
      "invite.send" => Some(Permission::SendInvite),
      "invite.view" => Some(Permission::ViewInvite),
      "user.remove" => Some(Permission::RemoveUser),
      "user.read" => Some(Permission::ReadUser),
      "guest.remove" => Some(Permission::RemoveGuest),
      "guest.read" => Some(Permission::ReadGuest),
      _ => None,
    }
  }

  /// Returns every permission defined in code.
  pub fn all() -> &'static [Permission] {
    &[
      Permission::ConfigureSettings,
      Permission::SendInvite,
      Permission::ViewInvite,
      Permission::RemoveUser,
      Permission::ReadUser,
      Permission::RemoveGuest,
      Permission::ReadGuest,
    ]
  }
}

impl std::fmt::Display for Permission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip_all_permissions() {
    for &p in Permission::all() {
      assert_eq!(Permission::from_code(p.as_str()), Some(p));
    }
  }

  #[test]
  fn unknown_code_returns_none() {
    assert_eq!(Permission::from_code("does.not.exist"), None);
  }
}
