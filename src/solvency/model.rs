use serde::Serialize;

use crate::{Amount, Bps, HorizonError, HorizonResult};

const BPS_SCALE: u128 = 10_000;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolvencyBand {
    Normal,
    Guarded,
    Restricted,
    Critical,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SolvencyPolicy {
    pub reserve_shock_bps: Bps,
    pub source_recovery_bps: Bps,
    pub backstop_recovery_bps: Bps,
    pub queued_coverage_bps: Bps,
    pub target_buffer_bps: Bps,
    pub max_route_concentration_bps: Bps,
    pub reserve_floor: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SolvencyInput {
    pub payout_reserve: Amount,
    pub source_reserve: Amount,
    pub live_routed_liability: Amount,
    pub queued_local_liability: Amount,
    pub queued_routed_liability: Amount,
    pub backstop_capacity: Amount,
    pub largest_route_liability: Amount,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SolvencyProjection {
    pub band: SolvencyBand,
    pub stressed_payout_reserve: Amount,
    pub eligible_source_reserve: Amount,
    pub eligible_backstop: Amount,
    pub eligible_liquidity: Amount,
    pub total_obligations: Amount,
    pub required_liquidity: Amount,
    pub target_liquidity: Amount,
    pub capital_buffer: Amount,
    pub capital_shortfall: Amount,
    pub coverage_bps: u32,
    pub route_concentration_bps: u16,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SolvencyModel;

impl SolvencyPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reserve_shock_bps: Bps,
        source_recovery_bps: Bps,
        backstop_recovery_bps: Bps,
        queued_coverage_bps: Bps,
        target_buffer_bps: Bps,
        max_route_concentration_bps: Bps,
        reserve_floor: Amount,
    ) -> HorizonResult<Self> {
        if max_route_concentration_bps.units() == 0 {
            return Err(HorizonError::Policy(
                "route concentration limit must be positive".to_owned(),
            ));
        }
        Ok(Self {
            reserve_shock_bps,
            source_recovery_bps,
            backstop_recovery_bps,
            queued_coverage_bps,
            target_buffer_bps,
            max_route_concentration_bps,
            reserve_floor,
        })
    }
}

impl SolvencyInput {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        payout_reserve: Amount,
        source_reserve: Amount,
        live_routed_liability: Amount,
        queued_local_liability: Amount,
        queued_routed_liability: Amount,
        backstop_capacity: Amount,
        largest_route_liability: Amount,
    ) -> Self {
        Self {
            payout_reserve,
            source_reserve,
            live_routed_liability,
            queued_local_liability,
            queued_routed_liability,
            backstop_capacity,
            largest_route_liability,
        }
    }
}

impl SolvencyModel {
    pub fn evaluate(
        policy: SolvencyPolicy,
        input: SolvencyInput,
    ) -> HorizonResult<SolvencyProjection> {
        let queued_liability = input
            .queued_local_liability
            .checked_add(input.queued_routed_liability)?;
        let total_obligations = input.live_routed_liability.checked_add(queued_liability)?;
        if input.largest_route_liability > total_obligations {
            return Err(HorizonError::Policy(
                "largest route exceeds total obligations".to_owned(),
            ));
        }

        let stressed_payout_reserve =
            retain_after_loss(input.payout_reserve, policy.reserve_shock_bps)?;
        let eligible_source_reserve = input
            .source_reserve
            .checked_mul_bps(policy.source_recovery_bps)?;
        let eligible_backstop = input
            .backstop_capacity
            .checked_mul_bps(policy.backstop_recovery_bps)?;
        let eligible_liquidity = stressed_payout_reserve
            .checked_add(eligible_source_reserve)?
            .checked_add(eligible_backstop)?;

        let covered_queue = queued_liability.checked_mul_bps(policy.queued_coverage_bps)?;
        let required_liquidity = input
            .live_routed_liability
            .checked_add(covered_queue)?
            .checked_add(policy.reserve_floor)?;
        let target_liquidity = required_liquidity
            .checked_add(required_liquidity.checked_mul_bps(policy.target_buffer_bps)?)?;

        let capital_shortfall = if eligible_liquidity < required_liquidity {
            required_liquidity.checked_sub(eligible_liquidity)?
        } else {
            Amount::zero()
        };
        let capital_buffer = if eligible_liquidity > required_liquidity {
            eligible_liquidity.checked_sub(required_liquidity)?
        } else {
            Amount::zero()
        };
        let coverage_bps = ratio_bps(eligible_liquidity, required_liquidity)?;
        let route_concentration_bps = if total_obligations.is_zero() {
            0
        } else {
            ratio_bps(input.largest_route_liability, total_obligations)?.min(10_000) as u16
        };

        let band = if !capital_shortfall.is_zero() {
            SolvencyBand::Critical
        } else if route_concentration_bps > policy.max_route_concentration_bps.units() {
            SolvencyBand::Restricted
        } else if eligible_liquidity < target_liquidity {
            SolvencyBand::Guarded
        } else {
            SolvencyBand::Normal
        };

        Ok(SolvencyProjection {
            band,
            stressed_payout_reserve,
            eligible_source_reserve,
            eligible_backstop,
            eligible_liquidity,
            total_obligations,
            required_liquidity,
            target_liquidity,
            capital_buffer,
            capital_shortfall,
            coverage_bps,
            route_concentration_bps,
        })
    }
}

fn retain_after_loss(amount: Amount, loss_bps: Bps) -> HorizonResult<Amount> {
    let retained_bps = 10_000_u16
        .checked_sub(loss_bps.units())
        .ok_or(HorizonError::BpsOutOfRange(loss_bps.units()))?;
    amount.checked_mul_bps(Bps::new(retained_bps)?)
}

fn ratio_bps(numerator: Amount, denominator: Amount) -> HorizonResult<u32> {
    if denominator.is_zero() {
        return Ok(10_000);
    }
    let ratio = numerator
        .units()
        .checked_mul(BPS_SCALE)
        .ok_or(HorizonError::AmountOverflow)?
        .checked_div(denominator.units())
        .ok_or(HorizonError::DivisionByZero)?;
    Ok(ratio.min(u128::from(u32::MAX)) as u32)
}
