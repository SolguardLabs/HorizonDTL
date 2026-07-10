use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, Bps, HorizonError, HorizonResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FeeSchedule {
    pub treasury: AccountId,
    pub protocol_fee_bps: Bps,
    pub reserve_fee_bps: Bps,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FeeLedger {
    schedule: Option<FeeSchedule>,
    accrued: BTreeMap<AssetId, Amount>,
    reserve_buffer: BTreeMap<AssetId, Amount>,
}

impl FeeSchedule {
    pub fn new(
        treasury: AccountId,
        protocol_fee_bps: Bps,
        reserve_fee_bps: Bps,
    ) -> HorizonResult<Self> {
        let total = u32::from(protocol_fee_bps.units()) + u32::from(reserve_fee_bps.units());
        if total > 10_000 {
            return Err(HorizonError::Policy(
                "fee schedule exceeds range".to_owned(),
            ));
        }
        Ok(Self {
            treasury,
            protocol_fee_bps,
            reserve_fee_bps,
        })
    }
}

impl FeeLedger {
    pub fn configure(&mut self, schedule: FeeSchedule) {
        self.schedule = Some(schedule);
    }

    pub fn protocol_fee(&self, notional: Amount) -> HorizonResult<Amount> {
        self.schedule
            .map(|schedule| notional.checked_mul_bps(schedule.protocol_fee_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn reserve_fee(&self, notional: Amount) -> HorizonResult<Amount> {
        self.schedule
            .map(|schedule| notional.checked_mul_bps(schedule.reserve_fee_bps))
            .unwrap_or_else(|| Ok(Amount::zero()))
    }

    pub fn accrue(&mut self, asset: AssetId, amount: Amount) -> HorizonResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let current = self
            .accrued
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero);
        self.accrued.insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn add_reserve_buffer(&mut self, asset: AssetId, amount: Amount) -> HorizonResult<()> {
        if amount.is_zero() {
            return Ok(());
        }
        let current = self
            .reserve_buffer
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero);
        self.reserve_buffer
            .insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn accrued_asset_count(&self) -> usize {
        self.accrued.len()
    }
}
