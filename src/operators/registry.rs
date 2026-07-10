use std::collections::BTreeMap;

use serde::Serialize;

use crate::{AccountId, Digest, HorizonError, HorizonResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Issuer,
    Beneficiary,
    Relayer,
    VaultController,
    Oracle,
    Treasury,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtocolConfig {
    pub admin: AccountId,
    pub current_epoch: u64,
    pub review_delay_epochs: u64,
    pub config_digest: Digest,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OperatorRegistry {
    roles: BTreeMap<AccountId, Vec<OperatorRole>>,
    config: Option<ProtocolConfig>,
    paused: bool,
}

impl ProtocolConfig {
    pub fn new(
        admin: AccountId,
        current_epoch: u64,
        review_delay_epochs: u64,
        salt: Digest,
    ) -> HorizonResult<Self> {
        let config_digest = Digest::from_serializable(
            "horizon-protocol-config-v1",
            &(admin, current_epoch, review_delay_epochs, salt),
        )?;
        Ok(Self {
            admin,
            current_epoch,
            review_delay_epochs,
            config_digest,
        })
    }
}

impl OperatorRegistry {
    pub fn configure(&mut self, config: ProtocolConfig) {
        self.config = Some(config);
    }

    pub fn grant_role(&mut self, account: AccountId, role: OperatorRole) {
        let roles = self.roles.entry(account).or_default();
        if !roles.contains(&role) {
            roles.push(role);
            roles.sort();
        }
    }

    pub fn require_role(&self, account: AccountId, role: OperatorRole) -> HorizonResult<()> {
        if self.has_role(account, role) {
            return Ok(());
        }
        Err(HorizonError::Policy(
            "operator role is not assigned".to_owned(),
        ))
    }

    pub fn has_role(&self, account: AccountId, role: OperatorRole) -> bool {
        self.roles
            .get(&account)
            .is_some_and(|roles| roles.contains(&role))
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn ensure_not_paused(&self) -> HorizonResult<()> {
        if self.paused {
            return Err(HorizonError::Policy("protocol is paused".to_owned()));
        }
        Ok(())
    }

    pub fn operator_count(&self) -> usize {
        self.roles.len()
    }
}
