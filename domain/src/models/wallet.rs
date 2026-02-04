use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::{ActorId, Id};

pub type WalletId = Id<Wallet>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletLabel {
  OutsideCash,
  OutsideCashDiscrepancy,
}

#[derive(Debug, Clone)]
pub struct Wallet {
  pub id: WalletId,
  pub owner: Option<ActorId>,
  pub label: Option<WalletLabel>,
  pub allow_overdraft: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

impl WalletLabel {
  pub fn variants() -> &'static [WalletLabel] {
    &[
      WalletLabel::OutsideCash,
      WalletLabel::OutsideCashDiscrepancy,
    ]
  }
}

impl Display for WalletLabel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let label_str = match self {
      WalletLabel::OutsideCash => "outside_cash",
      WalletLabel::OutsideCashDiscrepancy => "outside_cash_discrepancy",
    };
    write!(f, "{}", label_str)
  }
}

impl From<&str> for WalletLabel {
  fn from(value: &str) -> Self {
    match value {
      "outside_cash" => WalletLabel::OutsideCash,
      "outside_cash_discrepancy" => WalletLabel::OutsideCashDiscrepancy,
      _ => WalletLabel::OutsideCash,
    }
  }
}
