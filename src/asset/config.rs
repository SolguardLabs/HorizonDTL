use serde::Serialize;

use crate::{AssetId, Bps, HorizonError, HorizonResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AssetConfig {
    pub id: AssetId,
    pub symbol: &'static str,
    pub decimals: u8,
    pub max_fee_bps: Bps,
    pub settlement_enabled: bool,
}

impl AssetConfig {
    pub fn new(symbol: &'static str, decimals: u8, max_fee_bps: Bps) -> HorizonResult<Self> {
        if symbol.is_empty() || symbol.len() > 12 {
            return Err(HorizonError::Policy("asset symbol is invalid".to_owned()));
        }
        Ok(Self {
            id: AssetId::derive(symbol, decimals),
            symbol,
            decimals,
            max_fee_bps,
            settlement_enabled: true,
        })
    }

    pub const fn with_settlement(mut self, enabled: bool) -> Self {
        self.settlement_enabled = enabled;
        self
    }
}
