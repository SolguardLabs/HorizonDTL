use serde::{Deserialize, Serialize};

use crate::{HorizonError, HorizonResult};

const INDEX_SCALE: u128 = 1_000_000_000_000;

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Amount(u128);

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ShareUnits(u128);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShareIndex(u128);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bps(u16);

impl Amount {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub fn new(units: u128) -> HorizonResult<Self> {
        Ok(Self(units))
    }

    pub const fn units(self) -> u128 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Self) -> HorizonResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(HorizonError::AmountOverflow)
    }

    pub fn checked_sub(self, rhs: Self) -> HorizonResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(HorizonError::AmountUnderflow)
    }

    pub fn checked_mul(self, rhs: u128) -> HorizonResult<Self> {
        self.0
            .checked_mul(rhs)
            .map(Self)
            .ok_or(HorizonError::AmountOverflow)
    }

    pub fn checked_div(self, rhs: u128) -> HorizonResult<Self> {
        if rhs == 0 {
            return Err(HorizonError::DivisionByZero);
        }
        Ok(Self(self.0 / rhs))
    }

    pub fn checked_mul_bps(self, bps: Bps) -> HorizonResult<Self> {
        self.0
            .checked_mul(u128::from(bps.units()))
            .and_then(|value| value.checked_div(10_000))
            .map(Self)
            .ok_or(HorizonError::AmountOverflow)
    }
}

impl ShareUnits {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub fn new(units: u128) -> HorizonResult<Self> {
        Ok(Self(units))
    }

    pub const fn units(self) -> u128 {
        self.0
    }
}

impl ShareIndex {
    pub const fn one() -> Self {
        Self(INDEX_SCALE)
    }

    pub fn new(raw: u128) -> HorizonResult<Self> {
        if raw == 0 {
            return Err(HorizonError::DivisionByZero);
        }
        Ok(Self(raw))
    }

    pub const fn raw(self) -> u128 {
        self.0
    }

    pub fn units_from_amount(self, amount: Amount) -> HorizonResult<ShareUnits> {
        amount
            .units()
            .checked_mul(INDEX_SCALE)
            .and_then(|value| value.checked_div(self.0))
            .map(ShareUnits)
            .ok_or(HorizonError::AmountOverflow)
    }

    pub fn amount_from_units(self, units: ShareUnits) -> HorizonResult<Amount> {
        units
            .units()
            .checked_mul(self.0)
            .and_then(|value| value.checked_div(INDEX_SCALE))
            .map(Amount)
            .ok_or(HorizonError::AmountOverflow)
    }
}

impl Bps {
    pub fn new(units: u16) -> HorizonResult<Self> {
        if units > 10_000 {
            return Err(HorizonError::BpsOutOfRange(units));
        }
        Ok(Self(units))
    }

    pub const fn units(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::fmt::Display for ShareIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
