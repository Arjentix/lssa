use risc0_zkvm::sha::{Impl, Sha256};
use serde::{Deserialize, Serialize};

use crate::account::Commitment;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NullifierPublicKey(pub(super) [u8; 32]);

impl From<&NullifierSecretKey> for NullifierPublicKey {
    fn from(_value: &NullifierSecretKey) -> Self {
        todo!()
    }
}

pub type NullifierSecretKey = [u8; 32];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nullifier([u8; 32]);

impl Nullifier {
    pub fn new(commitment: &Commitment, nsk: &NullifierSecretKey) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&commitment.to_bytes());
        bytes.extend_from_slice(nsk);
        Self(Impl::hash_bytes(&bytes).as_bytes().try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let commitment = Commitment((0..32u8).collect::<Vec<_>>().try_into().unwrap());
        let nsk = [0x42; 32];
        let expected_nullifier = Nullifier([
            97, 87, 111, 191, 0, 44, 125, 145, 237, 104, 31, 230, 203, 254, 68, 176, 126, 17, 240,
            205, 249, 143, 11, 43, 15, 198, 189, 219, 191, 49, 36, 61,
        ]);
        let nullifier = Nullifier::new(&commitment, &nsk);
        assert_eq!(nullifier, expected_nullifier);
    }
}
