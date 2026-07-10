use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Amount, AssetId, HorizonError, HorizonResult, PublicIdentity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccountState {
    pub identity: PublicIdentity,
    pub balances: BTreeMap<AssetId, Amount>,
    pub next_ingress_nonce: u64,
    pub next_ticket_nonce: u64,
}

impl AccountState {
    pub fn new(identity: PublicIdentity) -> Self {
        Self {
            identity,
            balances: BTreeMap::new(),
            next_ingress_nonce: 0,
            next_ticket_nonce: 0,
        }
    }

    pub fn balance_of(&self, asset: AssetId) -> Amount {
        self.balances
            .get(&asset)
            .copied()
            .unwrap_or_else(Amount::zero)
    }

    pub(crate) fn credit(&mut self, asset: AssetId, amount: Amount) -> HorizonResult<()> {
        let next = self.balance_of(asset).checked_add(amount)?;
        self.set_balance(asset, next);
        Ok(())
    }

    pub(crate) fn debit(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> HorizonResult<()> {
        let available = self.balance_of(asset);
        if available < amount {
            return Err(HorizonError::InsufficientFunds {
                account,
                asset,
                available,
                required: amount,
            });
        }
        self.set_balance(asset, available.checked_sub(amount)?);
        Ok(())
    }

    pub(crate) fn advance_ingress_nonce(&mut self) -> HorizonResult<()> {
        self.next_ingress_nonce = self
            .next_ingress_nonce
            .checked_add(1)
            .ok_or(HorizonError::AmountOverflow)?;
        Ok(())
    }

    pub(crate) fn advance_ticket_nonce(&mut self) -> HorizonResult<()> {
        self.next_ticket_nonce = self
            .next_ticket_nonce
            .checked_add(1)
            .ok_or(HorizonError::AmountOverflow)?;
        Ok(())
    }

    fn set_balance(&mut self, asset: AssetId, amount: Amount) {
        if amount.is_zero() {
            self.balances.remove(&asset);
        } else {
            self.balances.insert(asset, amount);
        }
    }
}
