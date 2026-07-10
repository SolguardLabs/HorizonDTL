use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;

use crate::{AccountId, Digest, HorizonError, HorizonResult, SignatureBytes, canonical_bytes};

#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PublicIdentity {
    pub account: AccountId,
    pub verifying_key: [u8; 32],
}

impl KeyPair {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_identity(&self) -> PublicIdentity {
        let verifying_key = self.signing_key.verifying_key().to_bytes();
        let account = Digest::from_parts("horizon-account-v1", &[&verifying_key]).into();
        PublicIdentity {
            account,
            verifying_key,
        }
    }

    pub fn sign<T: Serialize>(&self, domain: &str, payload: &T) -> HorizonResult<SignatureBytes> {
        let bytes = canonical_bytes(&(domain, payload))?;
        Ok(SignatureBytes::from_signature(
            self.signing_key.sign(&bytes),
        ))
    }
}

impl PublicIdentity {
    pub fn verify_consistency(self) -> HorizonResult<()> {
        let expected: AccountId =
            Digest::from_parts("horizon-account-v1", &[&self.verifying_key]).into();
        if expected != self.account {
            return Err(HorizonError::IdentityMismatch(self.account));
        }
        Ok(())
    }
}
