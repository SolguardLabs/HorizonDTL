use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Bps, Digest, HorizonError, HorizonResult, NoteId, ShareIndex,
    VaultId,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VaultConfig {
    pub vault_id: VaultId,
    pub controller: AccountId,
    pub index_authority: AccountId,
    pub asset: AssetId,
    pub region: u16,
    pub reserve_floor: Amount,
    pub max_locked_bps: Bps,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VaultState {
    pub config: VaultConfig,
    pub reserve_balance: Amount,
    pub locked_notional: Amount,
    pub issued_notes: u64,
    pub settled_notes: u64,
    pub share_index: ShareIndex,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NoteRecord {
    pub note_id: NoteId,
    pub source_vault: VaultId,
    pub issuer: AccountId,
    pub settled: bool,
}

impl VaultConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        controller: AccountId,
        index_authority: AccountId,
        asset: AssetId,
        region: u16,
        reserve_floor: Amount,
        max_locked_bps: Bps,
        salt: Digest,
    ) -> Self {
        Self {
            vault_id: VaultId::derive(controller, asset, region, salt),
            controller,
            index_authority,
            asset,
            region,
            reserve_floor,
            max_locked_bps,
        }
    }
}

impl VaultState {
    pub fn new(config: VaultConfig) -> Self {
        Self {
            config,
            reserve_balance: Amount::zero(),
            locked_notional: Amount::zero(),
            issued_notes: 0,
            settled_notes: 0,
            share_index: ShareIndex::one(),
        }
    }

    pub fn deposit(&mut self, amount: Amount) -> HorizonResult<()> {
        if amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        self.reserve_balance = self.reserve_balance.checked_add(amount)?;
        Ok(())
    }

    pub fn lock_notional(&mut self, amount: Amount) -> HorizonResult<()> {
        let next_locked = self.locked_notional.checked_add(amount)?;
        if !self.locked_within_limit(next_locked)? {
            return Err(HorizonError::Policy(
                "vault locked notional exceeds limit".to_owned(),
            ));
        }
        self.locked_notional = next_locked;
        self.issued_notes = self
            .issued_notes
            .checked_add(1)
            .ok_or(HorizonError::AmountOverflow)?;
        Ok(())
    }

    pub fn release_notional(&mut self, amount: Amount) -> HorizonResult<()> {
        self.locked_notional = self.locked_notional.checked_sub(amount)?;
        self.settled_notes = self
            .settled_notes
            .checked_add(1)
            .ok_or(HorizonError::AmountOverflow)?;
        Ok(())
    }

    pub fn pay(&mut self, amount: Amount) -> HorizonResult<()> {
        let next_balance = self.reserve_balance.checked_sub(amount)?;
        if next_balance < self.config.reserve_floor {
            return Err(HorizonError::Policy(
                "vault reserve floor reached".to_owned(),
            ));
        }
        self.reserve_balance = next_balance;
        Ok(())
    }

    pub fn reprice(&mut self, share_index: ShareIndex) {
        self.share_index = share_index;
    }

    pub fn surplus(&self) -> HorizonResult<Amount> {
        let required = self
            .locked_notional
            .checked_add(self.config.reserve_floor)?;
        if self.reserve_balance <= required {
            return Ok(Amount::zero());
        }
        self.reserve_balance.checked_sub(required)
    }

    pub fn sweep(&mut self, amount: Amount) -> HorizonResult<()> {
        if amount > self.surplus()? {
            return Err(HorizonError::Policy(
                "vault surplus is below requested amount".to_owned(),
            ));
        }
        self.reserve_balance = self.reserve_balance.checked_sub(amount)?;
        Ok(())
    }

    fn locked_within_limit(&self, next_locked: Amount) -> HorizonResult<bool> {
        if self.reserve_balance.is_zero() {
            return Ok(false);
        }
        let ratio = next_locked
            .checked_mul(10_000)?
            .checked_div(self.reserve_balance.units())?;
        Ok(ratio.units() <= u128::from(self.config.max_locked_bps.units()))
    }
}
