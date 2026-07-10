use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Bps, Digest, NoteId, OperatorRole, ProtocolConfig, RiskSnapshot,
    ShareIndex, TicketId, TxId, VaultId,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    pub sequence: u64,
    pub tx_id: TxId,
    pub op: JournalOp,
    pub state_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JournalOp {
    GenesisCredit {
        account: AccountId,
        asset: AssetId,
        amount: Amount,
    },
    ProtocolConfigured {
        config: ProtocolConfig,
    },
    OperatorRoleGranted {
        account: AccountId,
        role: OperatorRole,
    },
    VaultRegistered {
        vault_id: VaultId,
        controller: AccountId,
        asset: AssetId,
    },
    VaultDeposit {
        vault_id: VaultId,
        owner: AccountId,
        amount: Amount,
    },
    VaultIndexPublished {
        vault_id: VaultId,
        observer: AccountId,
        share_index: ShareIndex,
        epoch: u64,
    },
    RouteRegistered {
        route_id: Digest,
        source_vault: VaultId,
        payout_vault: VaultId,
        asset: AssetId,
    },
    FeeScheduleConfigured {
        treasury: AccountId,
        protocol_fee_bps: Bps,
        reserve_fee_bps: Bps,
    },
    NoteIssued {
        note_id: NoteId,
        source_vault: VaultId,
        issuer: AccountId,
        beneficiary: AccountId,
        amount: Amount,
    },
    TicketSettled {
        ticket_id: TicketId,
        note_id: NoteId,
        route_id: Digest,
        source_vault: VaultId,
        payout_vault: VaultId,
        beneficiary: AccountId,
        beneficiary_amount: Amount,
        relayer: AccountId,
        relayer_fee: Amount,
        protocol_fee: Amount,
        reserve_fee: Amount,
        risk: Box<RiskSnapshot>,
    },
    VaultSurplusSwept {
        vault_id: VaultId,
        controller: AccountId,
        amount: Amount,
    },
}
