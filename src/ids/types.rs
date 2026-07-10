use serde::{Deserialize, Serialize, Serializer};

use crate::{Amount, HorizonResult, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct AccountId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct AssetId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct VaultId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct NoteId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct TicketId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct TxId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Digest([u8; 32]);

macro_rules! id_type {
    ($name:ident) => {
        impl $name {
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }

            pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> HorizonResult<Self> {
                let digest = Digest::from_serializable(domain, value)?;
                Ok(Self(digest.bytes()))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", hex::encode(self.0))
            }
        }
    };
}

id_type!(AccountId);
id_type!(AssetId);
id_type!(VaultId);
id_type!(NoteId);
id_type!(TicketId);
id_type!(TxId);

impl Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn from_parts(domain: &str, parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        for part in parts {
            hasher.update(&(part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
        Self(*hasher.finalize().as_bytes())
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> HorizonResult<Self> {
        let bytes = canonical_bytes(value)?;
        Ok(Self::from_parts(domain, &[&bytes]))
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", hex::encode(self.0))
    }
}

impl AssetId {
    pub fn derive(symbol: &str, decimals: u8) -> Self {
        Digest::from_parts("horizon-asset-v1", &[symbol.as_bytes(), &[decimals]]).into()
    }
}

impl VaultId {
    pub fn derive(controller: AccountId, asset: AssetId, region: u16, salt: Digest) -> Self {
        Digest::from_parts(
            "horizon-vault-v1",
            &[
                &controller.bytes(),
                &asset.bytes(),
                &region.to_be_bytes(),
                &salt.bytes(),
            ],
        )
        .into()
    }
}

impl NoteId {
    pub fn derive(
        network_id: u32,
        vault_id: VaultId,
        beneficiary: AccountId,
        asset: AssetId,
        amount: Amount,
        owner_nonce: u64,
        route_digest: Digest,
    ) -> Self {
        Digest::from_parts(
            "horizon-note-v1",
            &[
                &network_id.to_be_bytes(),
                &vault_id.bytes(),
                &beneficiary.bytes(),
                &asset.bytes(),
                &amount.units().to_be_bytes(),
                &owner_nonce.to_be_bytes(),
                &route_digest.bytes(),
            ],
        )
        .into()
    }
}

impl TicketId {
    pub fn derive(
        network_id: u32,
        payout_vault: VaultId,
        note_id: NoteId,
        relayer: AccountId,
        ticket_nonce: u64,
    ) -> Self {
        Digest::from_parts(
            "horizon-ticket-v1",
            &[
                &network_id.to_be_bytes(),
                &payout_vault.bytes(),
                &note_id.bytes(),
                &relayer.bytes(),
                &ticket_nonce.to_be_bytes(),
            ],
        )
        .into()
    }
}

impl From<Digest> for AccountId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for AssetId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for VaultId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for NoteId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for TicketId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for TxId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}
