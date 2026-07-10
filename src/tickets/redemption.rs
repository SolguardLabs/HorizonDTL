use serde::Serialize;

use crate::{
    AccountId, Amount, Digest, HorizonError, HorizonResult, KeyPair, NoteId, PublicIdentity,
    SignatureBytes, TicketId, TxId, VaultId, verify_signature,
};

pub const REDEMPTION_TICKET_DOMAIN: &str = "horizon-redemption-ticket-v1";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RedemptionTicket {
    pub network_id: u32,
    pub ticket_id: TicketId,
    pub payout_vault: VaultId,
    pub note_id: NoteId,
    pub beneficiary: AccountId,
    pub relayer: AccountId,
    pub relayer_fee: Amount,
    pub ticket_nonce: u64,
    pub settlement_epoch: u64,
    pub note_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RedemptionTicketAuthorizationView {
    network_id: u32,
    ticket_id: TicketId,
    payout_vault: VaultId,
    note_id: NoteId,
    beneficiary: AccountId,
    relayer: AccountId,
    relayer_fee: Amount,
    ticket_nonce: u64,
    settlement_epoch: u64,
    note_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedRedemptionTicket {
    pub signer: PublicIdentity,
    pub ticket: RedemptionTicket,
    pub signature: SignatureBytes,
}

impl RedemptionTicket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        payout_vault: VaultId,
        note_id: NoteId,
        beneficiary: AccountId,
        relayer: AccountId,
        relayer_fee: Amount,
        ticket_nonce: u64,
        settlement_epoch: u64,
        note_digest: Digest,
    ) -> HorizonResult<Self> {
        let ticket_id = TicketId::derive(network_id, payout_vault, note_id, relayer, ticket_nonce);
        Ok(Self {
            network_id,
            ticket_id,
            payout_vault,
            note_id,
            beneficiary,
            relayer,
            relayer_fee,
            ticket_nonce,
            settlement_epoch,
            note_digest,
        })
    }

    pub fn authorization_view(self) -> RedemptionTicketAuthorizationView {
        RedemptionTicketAuthorizationView {
            network_id: self.network_id,
            ticket_id: self.ticket_id,
            payout_vault: self.payout_vault,
            note_id: self.note_id,
            beneficiary: self.beneficiary,
            relayer: self.relayer,
            relayer_fee: self.relayer_fee,
            ticket_nonce: self.ticket_nonce,
            settlement_epoch: self.settlement_epoch,
            note_digest: self.note_digest,
        }
    }
}

impl SignedRedemptionTicket {
    pub fn sign(ticket: RedemptionTicket, key_pair: &KeyPair) -> HorizonResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != ticket.beneficiary {
            return Err(HorizonError::UnauthorizedSigner {
                expected: ticket.beneficiary,
                received: signer.account,
            });
        }
        let signature = key_pair.sign(REDEMPTION_TICKET_DOMAIN, &ticket.authorization_view())?;
        Ok(Self {
            signer,
            ticket,
            signature,
        })
    }

    pub fn verify(&self) -> HorizonResult<()> {
        if self.signer.account != self.ticket.beneficiary {
            return Err(HorizonError::UnauthorizedSigner {
                expected: self.ticket.beneficiary,
                received: self.signer.account,
            });
        }
        verify_signature(
            self.signer,
            self.signature,
            REDEMPTION_TICKET_DOMAIN,
            &self.ticket.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> HorizonResult<TxId> {
        TxId::from_serializable("horizon-signed-redemption-ticket-v1", self)
    }
}
