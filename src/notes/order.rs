use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Digest, HorizonError, HorizonResult, KeyPair, NoteId,
    PublicIdentity, ShareIndex, ShareUnits, SignatureBytes, TxId, VaultId, verify_signature,
};

pub const INGRESS_ORDER_DOMAIN: &str = "horizon-ingress-order-v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IngressOrder {
    pub network_id: u32,
    pub source_vault: VaultId,
    pub issuer: AccountId,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub owner_nonce: u64,
    pub maturity_epoch: u64,
    pub route_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IngressOrderAuthorizationView {
    network_id: u32,
    source_vault: VaultId,
    issuer: AccountId,
    beneficiary: AccountId,
    asset: AssetId,
    amount: Amount,
    owner_nonce: u64,
    maturity_epoch: u64,
    route_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeliveryNote {
    pub note_id: NoteId,
    pub network_id: u32,
    pub source_vault: VaultId,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub locked_units: ShareUnits,
    pub issued_index: ShareIndex,
    pub owner_nonce: u64,
    pub maturity_epoch: u64,
    pub route_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedIngressOrder {
    pub signer: PublicIdentity,
    pub order: IngressOrder,
    pub signature: SignatureBytes,
}

impl IngressOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        source_vault: VaultId,
        issuer: AccountId,
        beneficiary: AccountId,
        asset: AssetId,
        amount: Amount,
        owner_nonce: u64,
        maturity_epoch: u64,
        route_digest: Digest,
    ) -> HorizonResult<Self> {
        if amount.is_zero() {
            return Err(HorizonError::ZeroAmount);
        }
        Ok(Self {
            network_id,
            source_vault,
            issuer,
            beneficiary,
            asset,
            amount,
            owner_nonce,
            maturity_epoch,
            route_digest,
        })
    }

    pub fn authorization_view(self) -> IngressOrderAuthorizationView {
        IngressOrderAuthorizationView {
            network_id: self.network_id,
            source_vault: self.source_vault,
            issuer: self.issuer,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            owner_nonce: self.owner_nonce,
            maturity_epoch: self.maturity_epoch,
            route_digest: self.route_digest,
        }
    }

    pub fn note(self, issued_index: ShareIndex) -> HorizonResult<DeliveryNote> {
        let locked_units = issued_index.units_from_amount(self.amount)?;
        let note_id = NoteId::derive(
            self.network_id,
            self.source_vault,
            self.beneficiary,
            self.asset,
            self.amount,
            self.owner_nonce,
            self.route_digest,
        );
        Ok(DeliveryNote {
            note_id,
            network_id: self.network_id,
            source_vault: self.source_vault,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            locked_units,
            issued_index,
            owner_nonce: self.owner_nonce,
            maturity_epoch: self.maturity_epoch,
            route_digest: self.route_digest,
        })
    }
}

impl DeliveryNote {
    pub fn digest(self) -> HorizonResult<Digest> {
        Digest::from_serializable("horizon-delivery-note-v1", &self)
    }
}

impl SignedIngressOrder {
    pub fn sign(order: IngressOrder, key_pair: &KeyPair) -> HorizonResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != order.issuer {
            return Err(HorizonError::UnauthorizedSigner {
                expected: order.issuer,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(INGRESS_ORDER_DOMAIN, &order.authorization_view())?;
        Ok(Self {
            signer,
            order,
            signature,
        })
    }

    pub fn verify(&self) -> HorizonResult<()> {
        if self.signer.account != self.order.issuer {
            return Err(HorizonError::UnauthorizedSigner {
                expected: self.order.issuer,
                received: self.signer.account,
            });
        }
        verify_signature(
            self.signer,
            self.signature,
            INGRESS_ORDER_DOMAIN,
            &self.order.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> HorizonResult<TxId> {
        TxId::from_serializable("horizon-signed-ingress-order-v1", self)
    }
}
