mod encoding;
mod private_key;
mod public_key;
mod signature;

pub use private_key::PrivateKey;
pub use public_key::PublicKey;
pub use signature::Signature;

#[cfg(test)]
mod tests {
    use crate::{PrivateKey, PublicKey, Signature};

    fn hex_to_bytes<const N: usize>(hex: &str) -> [u8; N] {
        hex::decode(hex).unwrap().try_into().unwrap()
    }

    pub struct TestVector {
        pub seckey: Option<PrivateKey>,
        pub pubkey: PublicKey,
        pub aux_rand: Option<[u8; 32]>,
        pub message: Option<Vec<u8>>,
        pub signature: Signature,
        pub verification_result: bool,
    }

    /// Test vectors from
    /// https://github.com/bitcoin/bips/blob/master/bip-0340/test-vectors.csv
    //
    pub fn test_vectors() -> Vec<TestVector> {
        vec![
            TestVector {
                seckey: Some(PrivateKey(hex_to_bytes(
                    "0000000000000000000000000000000000000000000000000000000000000003",
                ))),
                pubkey: PublicKey(hex_to_bytes(
                    "F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9",
                )),
                aux_rand: Some(hex_to_bytes::<32>(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )),
                message: Some(
                    hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                        .unwrap(),
                ),
                signature: Signature {
                    value: hex_to_bytes(
                        "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA821525F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0",
                    ),
                },
                verification_result: true,
            },
            TestVector {
                seckey: Some(PrivateKey(hex_to_bytes(
                    "B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
                ))),
                pubkey: PublicKey(hex_to_bytes(
                    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
                )),
                aux_rand: Some(hex_to_bytes::<32>(
                    "0000000000000000000000000000000000000000000000000000000000000001",
                )),
                message: Some(
                    hex::decode("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89")
                        .unwrap(),
                ),
                signature: Signature {
                    value: hex_to_bytes(
                        "6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE33418906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A",
                    ),
                },
                verification_result: true,
            },
        ]
    }
}
