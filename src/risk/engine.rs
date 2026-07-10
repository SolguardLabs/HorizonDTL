use serde::Serialize;

use crate::{Amount, Bps, HorizonError, HorizonResult, TicketId, VaultId, VaultState};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskLimits {
    pub current_epoch: u64,
    pub max_ticket_amount: Amount,
    pub max_fee_bps: Bps,
    pub min_post_settlement_reserve: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RiskSnapshot {
    pub ticket_id: TicketId,
    pub vault_id: VaultId,
    pub projected_reserve: Amount,
    pub requested_amount: Amount,
    pub relayer_fee: Amount,
    pub fee_bps: Bps,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct RiskEngine {
    limits: RiskLimits,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            current_epoch: 0,
            max_ticket_amount: Amount::zero(),
            max_fee_bps: Bps::new(0).expect("zero bps is valid"),
            min_post_settlement_reserve: Amount::zero(),
        }
    }
}

impl RiskLimits {
    pub fn new(
        current_epoch: u64,
        max_ticket_amount: Amount,
        max_fee_bps: Bps,
        min_post_settlement_reserve: Amount,
    ) -> HorizonResult<Self> {
        if max_ticket_amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        Ok(Self {
            current_epoch,
            max_ticket_amount,
            max_fee_bps,
            min_post_settlement_reserve,
        })
    }
}

impl RiskEngine {
    pub fn set_limits(&mut self, limits: RiskLimits) {
        self.limits = limits;
    }

    pub const fn limits(&self) -> RiskLimits {
        self.limits
    }

    pub fn evaluate_ticket(
        &self,
        ticket_id: TicketId,
        vault: &VaultState,
        requested_amount: Amount,
        relayer_fee: Amount,
    ) -> HorizonResult<RiskSnapshot> {
        if requested_amount > self.limits.max_ticket_amount {
            return Err(HorizonError::Policy(
                "ticket amount exceeds risk limit".to_owned(),
            ));
        }
        let max_fee = requested_amount.checked_mul_bps(self.limits.max_fee_bps)?;
        if relayer_fee > max_fee {
            return Err(HorizonError::Policy(
                "relayer fee exceeds risk limit".to_owned(),
            ));
        }
        let projected_reserve = vault.reserve_balance.checked_sub(requested_amount)?;
        if projected_reserve < self.limits.min_post_settlement_reserve {
            return Err(HorizonError::Policy(
                "post settlement reserve is below limit".to_owned(),
            ));
        }
        let fee_bps = if requested_amount.is_zero() {
            Bps::new(0)?
        } else {
            Bps::new(((relayer_fee.units() * 10_000) / requested_amount.units()) as u16)?
        };
        Ok(RiskSnapshot {
            ticket_id,
            vault_id: vault.config.vault_id,
            projected_reserve,
            requested_amount,
            relayer_fee,
            fee_bps,
        })
    }
}
