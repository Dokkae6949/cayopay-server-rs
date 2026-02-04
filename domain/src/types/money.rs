use std::fmt;
use std::ops::{Add, Neg, Sub};

/// Money represented in minor currency units (cents)
///
/// Can be positive (credit) or negative (debt).
/// Stored as minor units (cents) internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(i64);

impl Money {
  /// Zero money
  pub const ZERO: Money = Money(0);

  /// Maximum representable money value
  pub const MAX: Money = Money(i64::MAX);

  /// Minimum representable money value (maximum debt)
  pub const MIN: Money = Money(i64::MIN);

  /// Create Money from minor units (cents)
  ///
  /// # Examples
  /// ```
  /// use domain::types::money::Money;
  /// let money = Money::from_minor(1050);
  /// assert_eq!(money.to_string(), "10.50");
  ///
  /// let debt = Money::from_minor(-1050);
  /// assert_eq!(debt.to_string(), "-10.50");
  /// ```
  pub const fn from_minor(cents: i64) -> Self {
    Self(cents)
  }

  /// Create Money from major units (euros)
  ///
  /// # Examples
  /// ```
  /// use domain::types::money::Money;
  /// let money = Money::from_major(10);
  /// assert_eq!(money.as_minor(), 1000);
  ///
  /// let debt = Money::from_major(-10);
  /// assert_eq!(debt.as_minor(), -1000);
  /// ```
  pub const fn from_major(euros: i64) -> Self {
    Self(euros.saturating_mul(100))
  }

  /// Get the raw minor units value (cents)
  pub const fn as_minor(&self) -> i64 {
    self.0
  }

  /// Get the major units (euros), preserving sign
  pub const fn as_major(&self) -> i64 {
    self.0 / 100
  }

  /// Get remaining cents after euros (always positive)
  ///
  /// For negative amounts, returns the absolute value of the remainder.
  /// For example, -10.50 returns 50 cents.
  pub const fn cents(&self) -> u64 {
    (self.0.saturating_abs() as u64) % 100
  }

  /// Format as currency string (e.g., "€10.50" or "€-10.50")
  pub fn format_eur(&self) -> String {
    if self.0 < 0 {
      format!("€-{}.{:02}", self.as_major().saturating_abs(), self.cents())
    } else {
      format!("€{}.{:02}", self.as_major(), self.cents())
    }
  }

  /// Check if the money amount is zero
  pub const fn is_zero(&self) -> bool {
    self.0 == 0
  }

  /// Check if the money amount is positive (credit)
  pub const fn is_positive(&self) -> bool {
    self.0 > 0
  }

  /// Check if the money amount is negative (debt)
  pub const fn is_negative(&self) -> bool {
    self.0 < 0
  }

  /// Get the absolute value
  pub const fn abs(&self) -> Self {
    Self(self.0.saturating_abs())
  }

  /// Checked addition. Returns `None` if overflow occurred.
  pub const fn checked_add(self, other: Self) -> Option<Self> {
    match self.0.checked_add(other.0) {
      Some(sum) => Some(Self(sum)),
      None => None,
    }
  }

  /// Checked subtraction. Returns `None` if overflow occurred.
  pub const fn checked_sub(self, other: Self) -> Option<Self> {
    match self.0.checked_sub(other.0) {
      Some(diff) => Some(Self(diff)),
      None => None,
    }
  }

  /// Saturating addition. Returns the max/min value on overflow.
  pub const fn saturating_add(self, other: Self) -> Self {
    Self(self.0.saturating_add(other.0))
  }

  /// Saturating subtraction. Returns the min/max value on overflow.
  pub const fn saturating_sub(self, other: Self) -> Self {
    Self(self.0.saturating_sub(other.0))
  }

  /// Checked negation. Returns `None` if negating would overflow.
  pub const fn checked_neg(self) -> Option<Self> {
    match self.0.checked_neg() {
      Some(neg) => Some(Self(neg)),
      None => None,
    }
  }
}

impl fmt::Display for Money {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.0 < 0 {
      write!(
        f,
        "-{}.{:02}",
        self.as_major().saturating_abs(),
        self.cents()
      )
    } else {
      write!(f, "{}.{:02}", self.as_major(), self.cents())
    }
  }
}

impl Default for Money {
  fn default() -> Self {
    Money::ZERO
  }
}

// Arithmetic operations
impl Add for Money {
  type Output = Money;

  fn add(self, other: Money) -> Money {
    self.saturating_add(other)
  }
}

impl Sub for Money {
  type Output = Money;

  fn sub(self, other: Money) -> Money {
    self.saturating_sub(other)
  }
}

impl Neg for Money {
  type Output = Money;

  fn neg(self) -> Money {
    Money(self.0.saturating_neg())
  }
}

// Database conversions
impl From<i64> for Money {
  fn from(value: i64) -> Self {
    Money(value)
  }
}

impl From<Money> for i64 {
  fn from(money: Money) -> Self {
    money.0
  }
}

impl TryFrom<u64> for Money {
  type Error = std::num::TryFromIntError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Ok(Money(i64::try_from(value)?))
  }
}

impl From<Money> for u64 {
  fn from(money: Money) -> Self {
    money.0.max(0) as u64
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ========================================================================
  // Construction Tests
  // ========================================================================

  #[test]
  fn test_from_minor_positive() {
    let money = Money::from_minor(1050);
    assert_eq!(money.as_minor(), 1050);
  }

  #[test]
  fn test_from_minor_negative() {
    let money = Money::from_minor(-1050);
    assert_eq!(money.as_minor(), -1050);
  }

  #[test]
  fn test_from_major_positive() {
    let money = Money::from_major(10);
    assert_eq!(money.as_minor(), 1000);
    assert_eq!(money.as_major(), 10);
  }

  #[test]
  fn test_from_major_negative() {
    let money = Money::from_major(-10);
    assert_eq!(money.as_minor(), -1000);
    assert_eq!(money.as_major(), -10);
  }

  #[test]
  fn test_from_major_saturating_mul() {
    // Test that saturating_mul prevents overflow in const context
    const LARGE: i64 = i64::MAX;
    const MONEY: Money = Money::from_major(LARGE);
    assert_eq!(MONEY.as_minor(), i64::MAX);

    const LARGE_NEG: i64 = i64::MIN;
    const MONEY_NEG: Money = Money::from_major(LARGE_NEG);
    assert_eq!(MONEY_NEG.as_minor(), i64::MIN);
  }

  #[test]
  fn test_zero_constant() {
    assert_eq!(Money::ZERO.as_minor(), 0);
    assert_eq!(Money::ZERO.as_major(), 0);
    assert!(Money::ZERO.is_zero());
    assert!(!Money::ZERO.is_positive());
    assert!(!Money::ZERO.is_negative());
  }

  #[test]
  fn test_min_max_constants() {
    assert_eq!(Money::MAX.as_minor(), i64::MAX);
    assert_eq!(Money::MIN.as_minor(), i64::MIN);
  }

  // ========================================================================
  // Accessor Tests
  // ========================================================================

  #[test]
  fn test_as_major_positive() {
    assert_eq!(Money::from_minor(1000).as_major(), 10);
    assert_eq!(Money::from_minor(1050).as_major(), 10);
    assert_eq!(Money::from_minor(99).as_major(), 0);
    assert_eq!(Money::from_minor(199).as_major(), 1);
  }

  #[test]
  fn test_as_major_negative() {
    assert_eq!(Money::from_minor(-1000).as_major(), -10);
    assert_eq!(Money::from_minor(-1050).as_major(), -10);
    assert_eq!(Money::from_minor(-99).as_major(), 0);
    assert_eq!(Money::from_minor(-199).as_major(), -1);
  }

  #[test]
  fn test_cents_positive() {
    assert_eq!(Money::from_minor(1050).cents(), 50);
    assert_eq!(Money::from_minor(1000).cents(), 0);
    assert_eq!(Money::from_minor(99).cents(), 99);
    assert_eq!(Money::from_minor(1).cents(), 1);
    assert_eq!(Money::from_minor(199).cents(), 99);
  }

  #[test]
  fn test_cents_negative() {
    assert_eq!(Money::from_minor(-1050).cents(), 50);
    assert_eq!(Money::from_minor(-1000).cents(), 0);
    assert_eq!(Money::from_minor(-99).cents(), 99);
    assert_eq!(Money::from_minor(-1).cents(), 1);
    assert_eq!(Money::from_minor(-199).cents(), 99);
  }

  #[test]
  fn test_is_zero() {
    assert!(Money::ZERO.is_zero());
    assert!(!Money::from_minor(1).is_zero());
    assert!(!Money::from_minor(-1).is_zero());
  }

  #[test]
  fn test_is_positive() {
    assert!(Money::from_minor(1).is_positive());
    assert!(Money::from_major(10).is_positive());
    assert!(!Money::ZERO.is_positive());
    assert!(!Money::from_minor(-1).is_positive());
  }

  #[test]
  fn test_is_negative() {
    assert!(Money::from_minor(-1).is_negative());
    assert!(Money::from_major(-10).is_negative());
    assert!(!Money::ZERO.is_negative());
    assert!(!Money::from_minor(1).is_negative());
  }

  #[test]
  fn test_abs() {
    assert_eq!(Money::from_minor(1050).abs(), Money::from_minor(1050));
    assert_eq!(Money::from_minor(-1050).abs(), Money::from_minor(1050));
    assert_eq!(Money::ZERO.abs(), Money::ZERO);
  }

  // ========================================================================
  // Display and Formatting Tests
  // ========================================================================

  #[test]
  fn test_display_positive() {
    assert_eq!(Money::from_minor(1050).to_string(), "10.50");
    assert_eq!(Money::from_minor(1000).to_string(), "10.00");
    assert_eq!(Money::from_minor(99).to_string(), "0.99");
    assert_eq!(Money::from_minor(1).to_string(), "0.01");
    assert_eq!(Money::ZERO.to_string(), "0.00");
  }

  #[test]
  fn test_display_negative() {
    assert_eq!(Money::from_minor(-1050).to_string(), "-10.50");
    assert_eq!(Money::from_minor(-1000).to_string(), "-10.00");
    assert_eq!(Money::from_minor(-99).to_string(), "-0.99");
    assert_eq!(Money::from_minor(-1).to_string(), "-0.01");
  }

  #[test]
  fn test_format_eur_positive() {
    assert_eq!(Money::from_minor(1050).format_eur(), "€10.50");
    assert_eq!(Money::from_minor(1000).format_eur(), "€10.00");
    assert_eq!(Money::from_minor(99).format_eur(), "€0.99");
    assert_eq!(Money::ZERO.format_eur(), "€0.00");
  }

  #[test]
  fn test_format_eur_negative() {
    assert_eq!(Money::from_minor(-1050).format_eur(), "€-10.50");
    assert_eq!(Money::from_minor(-1000).format_eur(), "€-10.00");
    assert_eq!(Money::from_minor(-99).format_eur(), "€-0.99");
    assert_eq!(Money::from_minor(-1).format_eur(), "€-0.01");
  }

  #[test]
  fn test_debug_format() {
    let money = Money::from_minor(1050);
    assert_eq!(format!("{:?}", money), "Money(1050)");

    let debt = Money::from_minor(-1050);
    assert_eq!(format!("{:?}", debt), "Money(-1050)");
  }

  // ========================================================================
  // Arithmetic Tests
  // ========================================================================

  #[test]
  fn test_addition_positive() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a + b, Money::from_minor(1500));
  }

  #[test]
  fn test_addition_negative() {
    let a = Money::from_minor(-1000);
    let b = Money::from_minor(-500);
    assert_eq!(a + b, Money::from_minor(-1500));
  }

  #[test]
  fn test_addition_mixed() {
    let positive = Money::from_minor(1000);
    let negative = Money::from_minor(-300);
    assert_eq!(positive + negative, Money::from_minor(700));
    assert_eq!(negative + positive, Money::from_minor(700));
  }

  #[test]
  fn test_subtraction_positive() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a - b, Money::from_minor(500));
  }

  #[test]
  fn test_subtraction_to_negative() {
    let a = Money::from_minor(500);
    let b = Money::from_minor(1000);
    assert_eq!(a - b, Money::from_minor(-500));
  }

  #[test]
  fn test_subtraction_negative() {
    let a = Money::from_minor(-1000);
    let b = Money::from_minor(500);
    assert_eq!(a - b, Money::from_minor(-1500));
  }

  #[test]
  fn test_negation() {
    assert_eq!(-Money::from_minor(1000), Money::from_minor(-1000));
    assert_eq!(-Money::from_minor(-1000), Money::from_minor(1000));
    assert_eq!(-Money::ZERO, Money::ZERO);
  }

  #[test]
  fn test_negation_min_saturates() {
    // i64::MIN cannot be negated without overflow, should saturate
    let min = Money::from_minor(i64::MIN);
    assert_eq!(-min, Money::from_minor(i64::MAX));
  }

  #[test]
  fn test_checked_negation() {
    assert_eq!(
      Money::from_minor(1000).checked_neg(),
      Some(Money::from_minor(-1000))
    );
    assert_eq!(
      Money::from_minor(-1000).checked_neg(),
      Some(Money::from_minor(1000))
    );
    assert_eq!(Money::from_minor(i64::MIN).checked_neg(), None);
  }

  #[test]
  fn test_addition_saturates_at_max() {
    let max = Money::from_minor(i64::MAX);
    let one = Money::from_minor(1);
    let result = max + one;
    assert_eq!(result.as_minor(), i64::MAX);
  }

  #[test]
  fn test_subtraction_saturates_at_min() {
    let min = Money::from_minor(i64::MIN);
    let one = Money::from_minor(1);
    let result = min - one;
    assert_eq!(result.as_minor(), i64::MIN);
  }

  #[test]
  fn test_checked_addition() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.checked_add(b), Some(Money::from_minor(1500)));

    let max = Money::from_minor(i64::MAX);
    let one = Money::from_minor(1);
    assert_eq!(max.checked_add(one), None);

    let min = Money::from_minor(i64::MIN);
    let neg_one = Money::from_minor(-1);
    assert_eq!(min.checked_add(neg_one), None);
  }

  #[test]
  fn test_checked_subtraction() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.checked_sub(b), Some(Money::from_minor(500)));

    let min = Money::from_minor(i64::MIN);
    let one = Money::from_minor(1);
    assert_eq!(min.checked_sub(one), None);

    let max = Money::from_minor(i64::MAX);
    let neg_one = Money::from_minor(-1);
    assert_eq!(max.checked_sub(neg_one), None);
  }

  // ========================================================================
  // Comparison Tests
  // ========================================================================

  #[test]
  fn test_ordering_with_negatives() {
    let debt = Money::from_minor(-1000);
    let zero = Money::ZERO;
    let credit = Money::from_minor(1000);

    assert!(debt < zero);
    assert!(zero < credit);
    assert!(debt < credit);
    assert!(credit > debt);
  }

  #[test]
  fn test_negative_comparisons() {
    let small_debt = Money::from_minor(-100);
    let large_debt = Money::from_minor(-1000);

    // Smaller negative is "larger" (less debt)
    assert!(small_debt > large_debt);
    assert!(large_debt < small_debt);
  }

  // ========================================================================
  // Database Conversion Tests
  // ========================================================================

  #[test]
  fn test_from_i64() {
    let money: Money = 1050i64.into();
    assert_eq!(money.as_minor(), 1050);

    let debt: Money = (-1050i64).into();
    assert_eq!(debt.as_minor(), -1050);
  }

  #[test]
  fn test_into_i64() {
    let money = Money::from_minor(1050);
    let value: i64 = money.into();
    assert_eq!(value, 1050);

    let debt = Money::from_minor(-1050);
    let value: i64 = debt.into();
    assert_eq!(value, -1050);
  }

  #[test]
  fn test_try_from_u64() {
    let money: Money = 1050u64.try_into().unwrap();
    assert_eq!(money.as_minor(), 1050);

    // u64::MAX cannot fit in i64
    let result: Result<Money, _> = u64::MAX.try_into();
    assert!(result.is_err());

    // Max i64 value should work
    let max_valid = i64::MAX as u64;
    let money: Money = max_valid.try_into().unwrap();
    assert_eq!(money.as_minor(), i64::MAX);
  }

  #[test]
  fn test_into_u64() {
    let money = Money::from_minor(1050);
    let value: u64 = money.into();
    assert_eq!(value, 1050);

    // Negative values become 0
    let debt = Money::from_minor(-1050);
    let value: u64 = debt.into();
    assert_eq!(value, 0);
  }

  // ========================================================================
  // Real-world Scenario Tests
  // ========================================================================

  #[test]
  fn test_overdraft_scenario() {
    let balance = Money::from_major(10);
    let withdrawal = Money::from_major(15);

    let new_balance = balance - withdrawal;
    assert_eq!(new_balance, Money::from_major(-5));
    assert!(new_balance.is_negative());
    assert_eq!(new_balance.format_eur(), "€-5.00");
  }

  #[test]
  fn test_debt_repayment() {
    let debt = Money::from_major(-100); // Owe €100
    let payment = Money::from_major(60); // Pay €60

    let remaining = debt + payment;
    assert_eq!(remaining, Money::from_major(-40)); // Still owe €40
    assert!(remaining.is_negative());
  }

  #[test]
  fn test_refund_scenario() {
    let balance = Money::from_major(50);
    let refund = Money::from_major(25);

    let new_balance = balance + refund;
    assert_eq!(new_balance, Money::from_major(75));
  }

  #[test]
  fn test_split_bill_with_debt() {
    let person_a = Money::from_major(20);
    let person_b = Money::from_major(-5); // Owes €5
    let total_bill = Money::from_major(30);

    let remaining_after_a = total_bill - person_a;
    let person_b_after = person_b - remaining_after_a;

    assert_eq!(person_b_after, Money::from_major(-15)); // Now owes €15
  }

  #[test]
  fn test_accounting_ledger() {
    let mut account = Money::ZERO;

    // Credit
    account = account + Money::from_minor(10000); // +€100.00
    assert_eq!(account, Money::from_major(100));

    // Debit
    account = account - Money::from_minor(15000); // -€150.00
    assert_eq!(account, Money::from_major(-50));
    assert!(account.is_negative());

    // Credit
    account = account + Money::from_minor(7500); // +€75.00
    assert_eq!(account, Money::from_major(25));
    assert!(account.is_positive());
  }

  // ========================================================================
  // Edge Case Tests
  // ========================================================================

  #[test]
  fn test_one_cent_debt() {
    let debt = Money::from_minor(-1);
    assert_eq!(debt.as_major(), 0);
    assert_eq!(debt.cents(), 1);
    assert_eq!(debt.to_string(), "-0.01");
    assert!(debt.is_negative());
  }

  #[test]
  fn test_boundary_values() {
    let max = Money::from_minor(i64::MAX);
    let min = Money::from_minor(i64::MIN);

    assert!(max.is_positive());
    assert!(min.is_negative());

    // These should not panic
    let _ = max.to_string();
    let _ = min.to_string();
    let _ = max.format_eur();
    let _ = min.format_eur();
  }

  // ========================================================================
  // Const Context Tests
  // ========================================================================

  #[test]
  fn test_const_functions_with_negatives() {
    const POSITIVE: Money = Money::from_major(10);
    const NEGATIVE: Money = Money::from_major(-10);
    const ZERO: Money = Money::ZERO;

    assert_eq!(POSITIVE.as_minor(), 1000);
    assert_eq!(NEGATIVE.as_minor(), -1000);
    assert_eq!(ZERO.as_minor(), 0);

    const ADDED: Money = POSITIVE.saturating_add(NEGATIVE);
    assert_eq!(ADDED.as_minor(), 0);
  }
}
