use std::fmt;
use std::ops::{Add, Sub};

/// Money represented in minor currency units (cents)
///
/// Always non-negative. Stored as minor units (cents) internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(u64);

impl Money {
  /// Zero money
  pub const ZERO: Money = Money(0);

  /// Create Money from minor units (cents)
  ///
  /// # Examples
  /// ```
  /// # use money::Money;
  /// let money = Money::from_minor(1050);
  /// assert_eq!(money.to_string(), "10.50");
  /// ```
  pub const fn from_minor(cents: u64) -> Self {
    Self(cents)
  }

  /// Create Money from major units (euros)
  ///
  /// # Examples
  /// ```
  /// # use money::Money;
  /// let money = Money::from_major(10);
  /// assert_eq!(money.as_minor(), 1000);
  /// ```
  pub const fn from_major(euros: u64) -> Self {
    Self(euros.saturating_mul(100))
  }

  /// Get the raw minor units value (cents)
  pub const fn as_minor(&self) -> u64 {
    self.0
  }

  /// Get the major units (euros)
  pub const fn as_major(&self) -> u64 {
    self.0 / 100
  }

  /// Get remaining cents after euros
  pub const fn cents(&self) -> u64 {
    self.0 % 100
  }

  /// Format as currency string (e.g., "€10.50")
  pub fn format_eur(&self) -> String {
    format!("€{}.{:02}", self.as_major(), self.cents())
  }

  /// Check if the money amount is zero
  pub const fn is_zero(&self) -> bool {
    self.0 == 0
  }

  /// Checked addition. Returns `None` if overflow occurred.
  pub const fn checked_add(self, other: Self) -> Option<Self> {
    match self.0.checked_add(other.0) {
      Some(sum) => Some(Self(sum)),
      None => None,
    }
  }

  /// Checked subtraction. Returns `None` if underflow occurred.
  pub const fn checked_sub(self, other: Self) -> Option<Self> {
    match self.0.checked_sub(other.0) {
      Some(diff) => Some(Self(diff)),
      None => None,
    }
  }

  /// Saturating addition. Returns the maximum value on overflow.
  pub const fn saturating_add(self, other: Self) -> Self {
    Self(self.0.saturating_add(other.0))
  }

  /// Saturating subtraction. Returns zero on underflow.
  pub const fn saturating_sub(self, other: Self) -> Self {
    Self(self.0.saturating_sub(other.0))
  }
}

impl fmt::Display for Money {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}.{:02}", self.as_major(), self.cents())
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

// Database conversions
impl From<i64> for Money {
  fn from(value: i64) -> Self {
    Money(value.max(0) as u64)
  }
}

impl From<Money> for i64 {
  fn from(money: Money) -> Self {
    money.0.min(i64::MAX as u64) as i64
  }
}

impl From<u64> for Money {
  fn from(value: u64) -> Self {
    Money(value)
  }
}

impl TryFrom<Money> for u64 {
  type Error = std::num::TryFromIntError;

  fn try_from(money: Money) -> Result<Self, Self::Error> {
    Ok(money.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ========================================================================
  // Construction Tests
  // ========================================================================

  #[test]
  fn test_from_minor() {
    let money = Money::from_minor(1050);
    assert_eq!(money.as_minor(), 1050);
  }

  #[test]
  fn test_from_major() {
    let money = Money::from_major(10);
    assert_eq!(money.as_minor(), 1000);
    assert_eq!(money.as_major(), 10);
  }

  #[test]
  fn test_from_major_saturating_mul() {
    // Test that saturating_mul prevents overflow in const context
    const LARGE: u64 = u64::MAX;
    const MONEY: Money = Money::from_major(LARGE);
    // This would overflow with regular multiplication
    assert_eq!(MONEY.as_minor(), u64::MAX);
  }

  #[test]
  fn test_zero_constant() {
    assert_eq!(Money::ZERO.as_minor(), 0);
    assert_eq!(Money::ZERO.as_major(), 0);
    assert!(Money::ZERO.is_zero());
  }

  #[test]
  fn test_from_major_zero() {
    let money = Money::from_major(0);
    assert_eq!(money, Money::ZERO);
  }

  // ========================================================================
  // Accessor Tests
  // ========================================================================

  #[test]
  fn test_as_major() {
    assert_eq!(Money::from_minor(1000).as_major(), 10);
    assert_eq!(Money::from_minor(1050).as_major(), 10);
    assert_eq!(Money::from_minor(99).as_major(), 0);
    assert_eq!(Money::from_minor(199).as_major(), 1);
  }

  #[test]
  fn test_cents() {
    assert_eq!(Money::from_minor(1050).cents(), 50);
    assert_eq!(Money::from_minor(1000).cents(), 0);
    assert_eq!(Money::from_minor(99).cents(), 99);
    assert_eq!(Money::from_minor(1).cents(), 1);
    assert_eq!(Money::from_minor(199).cents(), 99);
  }

  #[test]
  fn test_as_minor() {
    assert_eq!(Money::from_major(10).as_minor(), 1000);
    assert_eq!(Money::from_minor(1050).as_minor(), 1050);
  }

  #[test]
  fn test_is_zero() {
    assert!(Money::ZERO.is_zero());
    assert!(!Money::from_minor(1).is_zero());
    assert!(!Money::from_major(1).is_zero());
  }

  // ========================================================================
  // Display and Formatting Tests
  // ========================================================================

  #[test]
  fn test_display() {
    assert_eq!(Money::from_minor(1050).to_string(), "10.50");
    assert_eq!(Money::from_minor(1000).to_string(), "10.00");
    assert_eq!(Money::from_minor(99).to_string(), "0.99");
    assert_eq!(Money::from_minor(1).to_string(), "0.01");
    assert_eq!(Money::ZERO.to_string(), "0.00");
    assert_eq!(Money::from_minor(199).to_string(), "1.99");
  }

  #[test]
  fn test_format_eur() {
    assert_eq!(Money::from_minor(1050).format_eur(), "€10.50");
    assert_eq!(Money::from_minor(1000).format_eur(), "€10.00");
    assert_eq!(Money::from_minor(99).format_eur(), "€0.99");
    assert_eq!(Money::ZERO.format_eur(), "€0.00");
    assert_eq!(Money::from_minor(199).format_eur(), "€1.99");
  }

  #[test]
  fn test_format_large_amount() {
    let large = Money::from_major(1_000_000);
    assert_eq!(large.format_eur(), "€1000000.00");
  }

  #[test]
  fn test_debug_format() {
    let money = Money::from_minor(1050);
    assert_eq!(format!("{:?}", money), "Money(1050)");
    assert_eq!(format!("{:?}", Money::ZERO), "Money(0)");
  }

  // ========================================================================
  // Arithmetic Tests
  // ========================================================================

  #[test]
  fn test_addition() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a + b, Money::from_minor(1500));
  }

  #[test]
  fn test_addition_with_zero() {
    let money = Money::from_minor(1000);
    assert_eq!(money + Money::ZERO, money);
    assert_eq!(Money::ZERO + money, money);
  }

  #[test]
  fn test_subtraction() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a - b, Money::from_minor(500));
  }

  #[test]
  fn test_subtraction_to_zero() {
    let money = Money::from_minor(1000);
    assert_eq!(money - money, Money::ZERO);
  }

  #[test]
  fn test_subtraction_saturates_at_zero() {
    let small = Money::from_minor(500);
    let large = Money::from_minor(1000);

    // Should not panic, should saturate to zero
    assert_eq!(small - large, Money::ZERO);
  }

  #[test]
  fn test_addition_saturates_at_max() {
    let max = Money::from_minor(u64::MAX);
    let one = Money::from_minor(1);

    // Should not panic, should saturate
    let result = max + one;
    assert_eq!(result.as_minor(), u64::MAX);
  }

  #[test]
  fn test_addition_max_boundary() {
    let max_minus_one = Money::from_minor(u64::MAX - 1);
    let one = Money::from_minor(1);
    assert_eq!(max_minus_one + one, Money::from_minor(u64::MAX));
  }

  #[test]
  fn test_chained_operations() {
    let a = Money::from_major(10);
    let b = Money::from_major(5);
    let c = Money::from_major(3);

    let result = a + b - c;
    assert_eq!(result, Money::from_major(12));
  }

  #[test]
  fn test_checked_addition() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.checked_add(b), Some(Money::from_minor(1500)));

    let max = Money::from_minor(u64::MAX);
    let one = Money::from_minor(1);
    assert_eq!(max.checked_add(one), None);
  }

  #[test]
  fn test_checked_subtraction() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.checked_sub(b), Some(Money::from_minor(500)));

    let small = Money::from_minor(500);
    let large = Money::from_minor(1000);
    assert_eq!(small.checked_sub(large), None);
  }

  #[test]
  fn test_saturating_addition() {
    let max = Money::from_minor(u64::MAX);
    let one = Money::from_minor(1);
    assert_eq!(max.saturating_add(one), Money::from_minor(u64::MAX));

    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.saturating_add(b), Money::from_minor(1500));
  }

  #[test]
  fn test_saturating_subtraction() {
    let small = Money::from_minor(500);
    let large = Money::from_minor(1000);
    assert_eq!(small.saturating_sub(large), Money::ZERO);

    let a = Money::from_minor(1000);
    let b = Money::from_minor(500);
    assert_eq!(a.saturating_sub(b), Money::from_minor(500));
  }

  // ========================================================================
  // Comparison Tests
  // ========================================================================

  #[test]
  fn test_equality() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(1000);
    let c = Money::from_major(10);

    assert_eq!(a, b);
    assert_eq!(a, c);
  }

  #[test]
  fn test_inequality() {
    let a = Money::from_minor(1000);
    let b = Money::from_minor(2000);

    assert_ne!(a, b);
  }

  #[test]
  fn test_ordering() {
    let small = Money::from_minor(100);
    let medium = Money::from_minor(500);
    let large = Money::from_minor(1000);

    assert!(small < medium);
    assert!(medium < large);
    assert!(large > medium);
    assert!(medium > small);

    assert!(small <= medium);
    assert!(medium >= small);
    assert!(small <= small);
    assert!(small >= small);
  }

  #[test]
  fn test_zero_comparisons() {
    let money = Money::from_minor(100);

    assert!(money > Money::ZERO);
    assert!(Money::ZERO < money);
    assert!(Money::ZERO == Money::ZERO);
    assert!(Money::ZERO <= money);
    assert!(money >= Money::ZERO);
  }

  #[test]
  fn test_ord_consistency() {
    let a = Money::from_minor(100);
    let b = Money::from_minor(200);
    let c = Money::from_minor(300);

    // Transitivity
    assert!(a < b && b < c && a < c);

    // Antisymmetry
    assert!(!(a < b && b < a));

    // Totality - either a < b, a > b, or a == b
    assert!(a < b || a > b || a == b);
  }

  // ========================================================================
  // Database Conversion Tests
  // ========================================================================

  #[test]
  fn test_from_i64_positive() {
    let money: Money = 1050i64.into();
    assert_eq!(money.as_minor(), 1050);
  }

  #[test]
  fn test_from_i64_zero() {
    let money: Money = 0i64.into();
    assert_eq!(money, Money::ZERO);
  }

  #[test]
  fn test_from_i64_negative_becomes_zero() {
    let money: Money = (-100i64).into();
    assert_eq!(money, Money::ZERO);
  }

  #[test]
  fn test_into_i64() {
    let money = Money::from_minor(1050);
    let value: i64 = money.into();
    assert_eq!(value, 1050);
  }

  #[test]
  fn test_roundtrip_i64_conversion() {
    let original = 1050i64;
    let money: Money = original.into();
    let back: i64 = money.into();
    assert_eq!(original, back);
  }

  #[test]
  fn test_from_i64_large_value() {
    let large = i64::MAX;
    let money: Money = large.into();
    let back: i64 = money.into();
    assert_eq!(back, large);
  }

  #[test]
  fn test_from_u64() {
    let money: Money = 1050u64.into();
    assert_eq!(money.as_minor(), 1050);
  }

  #[test]
  fn test_try_into_u64() {
    let money = Money::from_minor(1050);
    let value: u64 = money.try_into().unwrap();
    assert_eq!(value, 1050);

    // Maximum value should convert fine
    let max_money = Money::from_minor(u64::MAX);
    let max_value: u64 = max_money.try_into().unwrap();
    assert_eq!(max_value, u64::MAX);
  }

  // ========================================================================
  // Default Tests
  // ========================================================================

  #[test]
  fn test_default() {
    let money: Money = Default::default();
    assert_eq!(money, Money::ZERO);
  }

  // ========================================================================
  // Const Context Tests
  // ========================================================================

  #[test]
  fn test_const_functions() {
    // Verify these can be used in const contexts
    const MONEY: Money = Money::from_major(10);
    const ZERO: Money = Money::ZERO;
    const FROM_MINOR: Money = Money::from_minor(1050);

    assert_eq!(MONEY.as_minor(), 1000);
    assert_eq!(ZERO.as_minor(), 0);
    assert_eq!(FROM_MINOR.as_minor(), 1050);

    // Test const operations
    const ADDED: Money = MONEY.saturating_add(ZERO);
    const SUBTRACTED: Money = MONEY.saturating_sub(Money::from_major(5));

    assert_eq!(ADDED.as_minor(), 1000);
    assert_eq!(SUBTRACTED.as_minor(), 500);
  }

  // ========================================================================
  // Edge Case Tests
  // ========================================================================

  #[test]
  fn test_one_cent() {
    let money = Money::from_minor(1);
    assert_eq!(money.as_major(), 0);
    assert_eq!(money.cents(), 1);
    assert_eq!(money.to_string(), "0.01");
    assert!(!money.is_zero());
  }

  #[test]
  fn test_ninety_nine_cents() {
    let money = Money::from_minor(99);
    assert_eq!(money.as_major(), 0);
    assert_eq!(money.cents(), 99);
    assert_eq!(money.to_string(), "0.99");
  }

  #[test]
  fn test_exact_euros() {
    let money = Money::from_major(50);
    assert_eq!(money.as_major(), 50);
    assert_eq!(money.cents(), 0);
    assert_eq!(money.to_string(), "50.00");
  }

  #[test]
  fn test_large_amount() {
    let money = Money::from_major(1_000_000);
    assert_eq!(money.as_major(), 1_000_000);
    assert_eq!(money.as_minor(), 100_000_000);
  }

  #[test]
  fn test_max_amount() {
    let money = Money::from_minor(u64::MAX);
    assert_eq!(money.as_minor(), u64::MAX);
    // This should not panic
    let _ = money.as_major();
    let _ = money.cents();
    let _ = money.to_string();
  }

  // ========================================================================
  // Real-world Scenario Tests
  // ========================================================================

  #[test]
  fn test_restaurant_bill_scenario() {
    let meal = Money::from_minor(2450); // €24.50
    let drink = Money::from_minor(350); // €3.50
    let dessert = Money::from_minor(650); // €6.50

    let subtotal = meal + drink + dessert;
    assert_eq!(subtotal.as_minor(), 3450); // €34.50
    assert_eq!(subtotal.format_eur(), "€34.50");
  }

  #[test]
  fn test_change_calculation() {
    let price = Money::from_minor(1275); // €12.75
    let paid = Money::from_major(20); // €20.00

    let change = paid - price;
    assert_eq!(change.as_minor(), 725); // €7.25
    assert_eq!(change.format_eur(), "€7.25");
  }

  #[test]
  fn test_insufficient_funds() {
    let balance = Money::from_major(10);
    let price = Money::from_major(15);

    // Should not panic, saturates to zero
    let remaining = balance - price;
    assert_eq!(remaining, Money::ZERO);
  }

  #[test]
  fn test_multiple_small_transactions() {
    let mut balance = Money::from_major(100);

    balance = balance - Money::from_minor(99); // €0.99
    balance = balance - Money::from_minor(150); // €1.50
    balance = balance - Money::from_minor(251); // €2.51

    assert_eq!(balance, Money::from_major(95)); // €95.00
  }

  #[test]
  fn test_bank_account_transactions() {
    let mut account = Money::from_major(1000); // €1000.00

    // Deposits
    account = account + Money::from_minor(4999); // €49.99
    account = account + Money::from_major(500); // €500.00

    // Withdrawals
    account = account - Money::from_minor(10000); // €100.00
    account = account - Money::from_major(200); // €200.00

    // Attempt overdraw (should saturate to zero)
    account = account - Money::from_major(2000);

    assert_eq!(account, Money::ZERO);
  }

  // ========================================================================
  // Copy and Clone Tests
  // ========================================================================

  #[test]
  fn test_copy() {
    let original = Money::from_major(10);
    let copied = original;

    // Both should be usable
    assert_eq!(original, copied);
    assert_eq!(original.as_minor(), 1000);
    assert_eq!(copied.as_minor(), 1000);
  }

  #[test]
  fn test_clone() {
    let original = Money::from_major(10);
    let cloned = original.clone();

    assert_eq!(original, cloned);
  }

  // ========================================================================
  // Hash Tests
  // ========================================================================

  #[test]
  fn test_hash_consistency() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let key = Money::from_major(10);

    map.insert(key, "ten euros");

    assert_eq!(map.get(&Money::from_minor(1000)), Some(&"ten euros"));
    assert_eq!(map.get(&Money::from_major(10)), Some(&"ten euros"));
    assert_eq!(map.get(&Money::ZERO), None);
  }

  // ========================================================================
  // Doc Tests
  // ========================================================================

  #[test]
  fn test_doc_examples() {
    // Test examples from doc comments
    let money = Money::from_minor(1050);
    assert_eq!(money.to_string(), "10.50");

    let money = Money::from_major(10);
    assert_eq!(money.as_minor(), 1000);
  }
}
