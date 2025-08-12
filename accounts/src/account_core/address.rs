use common::transaction::{SignaturePublicKey, Tag};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub struct AccountAddress(pub(crate) [u8; 32]);

impl AccountAddress {
    pub fn new(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub fn tag(&self) -> Tag {
        self.0[0]
    }

    pub fn raw_addr(&self) -> [u8; 32] {
        self.0
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = Vec<u8>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let addr_val: [u8; 32] = value.try_into()?;

        Ok(AccountAddress::new(addr_val))
    }
}

impl From<&SignaturePublicKey> for AccountAddress {
    fn from(value: &SignaturePublicKey) -> Self {
        let mut address = [0; 32];
        let mut keccak_hasher = Keccak::v256();
        keccak_hasher.update(&value.to_sec1_bytes());
        keccak_hasher.finalize(&mut address);
        AccountAddress::new(address)
    }
}

#[cfg(test)]
mod tests {
    use common::transaction::SignaturePrivateKey;

    use super::*;

    #[test]
    fn test_address_key_equal_keccak_pub_sign_key() {
        let signing_key = SignaturePrivateKey::from_slice(&[1; 32]).unwrap();
        let public_key = signing_key.verifying_key();

        let mut expected_address = [0; 32];
        let mut keccak_hasher = Keccak::v256();
        keccak_hasher.update(&public_key.to_sec1_bytes());
        keccak_hasher.finalize(&mut expected_address);

        assert_eq!(
            AccountAddress::new(expected_address),
            AccountAddress::from(public_key)
        );
    }
}
