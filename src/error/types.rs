use thiserror::Error;

use crate::{AccountId, Amount, AssetId, NoteId, TicketId, TxId, VaultId};

pub type HorizonResult<T> = Result<T, HorizonError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HorizonError {
    #[error("amount overflow")]
    AmountOverflow,
    #[error("amount underflow")]
    AmountUnderflow,
    #[error("division by zero")]
    DivisionByZero,
    #[error("zero amount")]
    ZeroAmount,
    #[error("basis points out of range: {0}")]
    BpsOutOfRange(u16),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("signature error: {0}")]
    Signature(String),
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("identity mismatch: {0}")]
    IdentityMismatch(AccountId),
    #[error("account not found: {0}")]
    AccountNotFound(AccountId),
    #[error("account already exists: {0}")]
    AccountAlreadyExists(AccountId),
    #[error("asset not found: {0}")]
    AssetNotFound(AssetId),
    #[error("asset already exists: {0}")]
    AssetAlreadyExists(AssetId),
    #[error("vault not found: {0}")]
    VaultNotFound(VaultId),
    #[error("vault already exists: {0}")]
    VaultAlreadyExists(VaultId),
    #[error("note not found: {0}")]
    NoteNotFound(NoteId),
    #[error("note already exists: {0}")]
    NoteAlreadyExists(NoteId),
    #[error("note already settled: {0}")]
    NoteSettled(NoteId),
    #[error("ticket already processed: {0}")]
    TicketProcessed(TicketId),
    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(TxId),
    #[error("unauthorized signer: expected {expected}, received {received}")]
    UnauthorizedSigner {
        expected: AccountId,
        received: AccountId,
    },
    #[error("nonce mismatch for {account}: expected {expected}, received {received}")]
    NonceMismatch {
        account: AccountId,
        expected: u64,
        received: u64,
    },
    #[error(
        "insufficient funds for {account} on {asset}: available {available}, required {required}"
    )]
    InsufficientFunds {
        account: AccountId,
        asset: AssetId,
        available: Amount,
        required: Amount,
    },
    #[error("policy violation: {0}")]
    Policy(String),
    #[error("conservation error for {asset}: expected {expected}, observed {observed}")]
    Conservation {
        asset: AssetId,
        expected: Amount,
        observed: Amount,
    },
}
