use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, AssetId, Bps, Digest, HorizonResult, ShareIndex, VaultId};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IndexObservation {
    pub vault_id: VaultId,
    pub asset: AssetId,
    pub observer: AccountId,
    pub share_index: ShareIndex,
    pub confidence_bps: Bps,
    pub epoch: u64,
    pub observation_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OracleBook {
    observations: BTreeMap<VaultId, IndexObservation>,
}

impl IndexObservation {
    pub fn new(
        vault_id: VaultId,
        asset: AssetId,
        observer: AccountId,
        share_index: ShareIndex,
        confidence_bps: Bps,
        epoch: u64,
    ) -> HorizonResult<Self> {
        let observation_digest = Digest::from_serializable(
            "horizon-index-observation-v1",
            &(
                vault_id,
                asset,
                observer,
                share_index,
                confidence_bps,
                epoch,
            ),
        )?;
        Ok(Self {
            vault_id,
            asset,
            observer,
            share_index,
            confidence_bps,
            epoch,
            observation_digest,
        })
    }
}

impl OracleBook {
    pub fn publish(&mut self, observation: IndexObservation) {
        self.observations.insert(observation.vault_id, observation);
    }

    pub fn latest(&self, vault_id: VaultId) -> Option<IndexObservation> {
        self.observations.get(&vault_id).copied()
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}
