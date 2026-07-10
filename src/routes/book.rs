use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Amount, AssetId, Bps, Digest, HorizonError, HorizonResult, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SettlementRoute {
    pub route_id: Digest,
    pub source_vault: VaultId,
    pub payout_vault: VaultId,
    pub asset: AssetId,
    pub min_amount: Amount,
    pub max_amount: Amount,
    pub max_relayer_fee_bps: Bps,
    pub finality_delay_epochs: u64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct RouteBook {
    routes: BTreeMap<Digest, SettlementRoute>,
}

impl SettlementRoute {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_vault: VaultId,
        payout_vault: VaultId,
        asset: AssetId,
        min_amount: Amount,
        max_amount: Amount,
        max_relayer_fee_bps: Bps,
        finality_delay_epochs: u64,
        salt: Digest,
    ) -> HorizonResult<Self> {
        if min_amount.is_zero() || max_amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        if min_amount > max_amount {
            return Err(HorizonError::Policy(
                "route amount range is invalid".to_owned(),
            ));
        }
        let route_id = Digest::from_serializable(
            "horizon-route-v1",
            &(
                source_vault,
                payout_vault,
                asset,
                min_amount,
                max_amount,
                max_relayer_fee_bps,
                finality_delay_epochs,
                salt,
            ),
        )?;
        Ok(Self {
            route_id,
            source_vault,
            payout_vault,
            asset,
            min_amount,
            max_amount,
            max_relayer_fee_bps,
            finality_delay_epochs,
            enabled: true,
        })
    }

    pub fn accepts(
        self,
        source_vault: VaultId,
        payout_vault: VaultId,
        asset: AssetId,
        amount: Amount,
        relayer_fee: Amount,
    ) -> HorizonResult<bool> {
        if !self.enabled {
            return Ok(false);
        }
        if self.source_vault != source_vault
            || self.payout_vault != payout_vault
            || self.asset != asset
        {
            return Ok(false);
        }
        if amount < self.min_amount || amount > self.max_amount {
            return Ok(false);
        }
        let max_fee = amount.checked_mul_bps(self.max_relayer_fee_bps)?;
        Ok(relayer_fee <= max_fee)
    }
}

impl RouteBook {
    pub fn register_route(&mut self, route: SettlementRoute) -> HorizonResult<Digest> {
        if self.routes.contains_key(&route.route_id) {
            return Err(HorizonError::Policy("route already exists".to_owned()));
        }
        self.routes.insert(route.route_id, route);
        Ok(route.route_id)
    }

    pub fn resolve_route(
        &self,
        source_vault: VaultId,
        payout_vault: VaultId,
        asset: AssetId,
        amount: Amount,
        relayer_fee: Amount,
    ) -> HorizonResult<SettlementRoute> {
        for route in self.routes.values().copied() {
            if route.accepts(source_vault, payout_vault, asset, amount, relayer_fee)? {
                return Ok(route);
            }
        }
        Err(HorizonError::Policy(
            "settlement route not available".to_owned(),
        ))
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}
