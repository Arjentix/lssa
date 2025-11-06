use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::key_management::secret_holders::SeedHolder;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ChainIndex(Vec<u32>);

impl FromStr for ChainIndex {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self(vec![]));
        }

        let hex_decoded = hex::decode(s)?;

        if !hex_decoded.len().is_multiple_of(4) {
            Err(hex::FromHexError::InvalidStringLength)
        } else {
            let mut res_vec = vec![];

            for i in 0..(hex_decoded.len() / 4) {
                res_vec.push(u32::from_le_bytes([
                    hex_decoded[4 * i],
                    hex_decoded[4 * i + 1],
                    hex_decoded[4 * i + 2],
                    hex_decoded[4 * i + 3],
                ]));
            }

            Ok(Self(res_vec))
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ChainIndex {
    fn to_string(&self) -> String {
        if self.0.is_empty() {
            return "".to_string();
        }

        let mut res_vec = vec![];

        for index in &self.0 {
            res_vec.extend_from_slice(&index.to_le_bytes());
        }

        hex::encode(res_vec)
    }
}

impl ChainIndex {
    pub fn root() -> Self {
        ChainIndex::from_str("").unwrap()
    }

    pub fn chain(&self) -> &[u32] {
        &self.0
    }

    pub fn next_in_line(&self) -> ChainIndex {
        let mut chain = self.0.clone();
        //ToDo: Add overflow check
        if let Some(last_p) = chain.last_mut() {
            *last_p += 1
        }

        ChainIndex(chain)
    }

    pub fn n_th_child(&self, child_id: u32) -> ChainIndex {
        let mut chain = self.0.clone();
        chain.push(child_id);

        ChainIndex(chain)
    }
}

#[derive(Debug)]
pub struct ChildKeysPublic {
    pub csk: nssa::PrivateKey,
    pub cpk: nssa::PublicKey,
    pub ccc: [u8; 32],
    ///Can be None if root
    pub cci: Option<u32>,
}

impl ChildKeysPublic {
    pub fn root(seed: [u8; 64]) -> Self {
        let hash_value = hmac_sha512::HMAC::mac(seed, "NSSA_master_pub");

        let csk = nssa::PrivateKey::try_new(*hash_value.first_chunk::<32>().unwrap()).unwrap();
        let ccc = *hash_value.last_chunk::<32>().unwrap();
        let cpk = nssa::PublicKey::new_from_private_key(&csk);

        Self {
            csk,
            cpk,
            ccc,
            cci: None,
        }
    }

    pub fn n_th_child(&self, cci: u32) -> Self {
        let mut hash_input = vec![];
        hash_input.extend_from_slice(self.csk.value());
        hash_input.extend_from_slice(&cci.to_le_bytes());

        let hash_value = hmac_sha512::HMAC::mac(&hash_input, self.ccc);

        let csk = nssa::PrivateKey::try_new(*hash_value.first_chunk::<32>().unwrap()).unwrap();
        let ccc = *hash_value.last_chunk::<32>().unwrap();
        let cpk = nssa::PublicKey::new_from_private_key(&csk);

        Self {
            csk,
            cpk,
            ccc,
            cci: Some(cci),
        }
    }
}

#[derive(Debug)]
pub struct KeyTreePublic {
    pub key_map: BTreeMap<ChainIndex, ChildKeysPublic>,
    pub addr_map: HashMap<nssa::Address, ChainIndex>,
}

impl KeyTreePublic {
    pub fn new(seed: &SeedHolder) -> Self {
        let seed_fit: [u8; 64] = seed.seed.clone().try_into().unwrap();

        let root_keys = ChildKeysPublic::root(seed_fit);
        let address = nssa::Address::from(&root_keys.cpk);

        let mut key_map = BTreeMap::new();
        let mut addr_map = HashMap::new();

        key_map.insert(ChainIndex::root(), root_keys);
        addr_map.insert(address, ChainIndex::root());

        Self { key_map, addr_map }
    }

    pub fn find_next_last_child_of_id(&self, parent_id: &ChainIndex) -> Option<u32> {
        if !self.key_map.contains_key(parent_id) {
            return None;
        }

        let leftmost_child = parent_id.n_th_child(u32::MIN);

        if !self.key_map.contains_key(&leftmost_child) {
            Some(0)
        } else {
            let mut right = u32::MAX - 1;
            let mut left_border = u32::MIN;
            let mut right_border = u32::MAX;

            loop {
                let rightmost_child = parent_id.n_th_child(right);

                let rightmost_ref = self.key_map.get(&rightmost_child);
                let rightmost_ref_next = self.key_map.get(&rightmost_child.next_in_line());

                match (&rightmost_ref, &rightmost_ref_next) {
                    (Some(_), Some(_)) => {
                        left_border = right;
                        right = (right + right_border) / 2;
                    }
                    (Some(_), None) => {
                        break Some(right + 1);
                    }
                    (None, None) => {
                        right_border = right;
                        right = (left_border + right) / 2;
                    }
                    (None, Some(_)) => {
                        unreachable!();
                    }
                }
            }
        }
    }

    pub fn generate_new_pub_keys(&mut self, parent_cci: ChainIndex) -> Option<nssa::Address> {
        if !self.key_map.contains_key(&parent_cci) {
            return None;
        }

        let father_keys = self.key_map.get(&parent_cci).unwrap();
        let next_child_id = self.find_next_last_child_of_id(&parent_cci).unwrap();
        let next_cci = parent_cci.n_th_child(next_child_id);

        let child_keys = father_keys.n_th_child(next_child_id);

        let address = nssa::Address::from(&child_keys.cpk);

        self.key_map.insert(next_cci.clone(), child_keys);
        self.addr_map.insert(address, next_cci);

        Some(address)
    }

    pub fn get_pub_keys(&self, addr: nssa::Address) -> Option<&ChildKeysPublic> {
        self.addr_map
            .get(&addr)
            .and_then(|chain_id| self.key_map.get(chain_id))
    }
}

#[cfg(test)]
mod tests {
    use nssa::Address;

    use super::*;

    #[test]
    fn test_chain_id_root_correct() {
        let chain_id = ChainIndex::root();
        let chain_id_2 = ChainIndex::from_str("").unwrap();

        assert_eq!(chain_id, chain_id_2);
    }

    #[test]
    fn test_chain_id_deser_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();

        assert_eq!(chain_id.chain(), &[257]);
    }

    #[test]
    fn test_chain_id_next_in_line_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();
        let next_in_line = chain_id.next_in_line();

        assert_eq!(next_in_line, ChainIndex::from_str("02010000").unwrap());
    }

    #[test]
    fn test_chain_id_child_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();
        let child = chain_id.n_th_child(3);

        assert_eq!(child, ChainIndex::from_str("0101000003000000").unwrap());
    }

    #[test]
    fn test_keys_deterministic_generation() {
        let root_keys = ChildKeysPublic::root([42; 64]);
        let child_keys = root_keys.n_th_child(5);

        assert_eq!(root_keys.cci, None);
        assert_eq!(child_keys.cci, Some(5));

        assert_eq!(
            root_keys.ccc,
            [
                61, 30, 91, 26, 133, 91, 236, 192, 231, 53, 186, 139, 11, 221, 202, 11, 178, 215,
                254, 103, 191, 60, 117, 112, 1, 226, 31, 156, 83, 104, 150, 224
            ]
        );
        assert_eq!(
            child_keys.ccc,
            [
                67, 26, 102, 68, 189, 155, 102, 80, 199, 188, 112, 142, 207, 157, 36, 210, 48, 224,
                35, 6, 112, 180, 11, 190, 135, 218, 9, 14, 84, 231, 58, 98
            ]
        );

        assert_eq!(
            root_keys.csk.value(),
            &[
                241, 82, 246, 237, 62, 130, 116, 47, 189, 112, 99, 67, 178, 40, 115, 245, 141, 193,
                77, 164, 243, 76, 222, 64, 50, 146, 23, 145, 91, 164, 92, 116
            ]
        );
        assert_eq!(
            child_keys.csk.value(),
            &[
                11, 151, 27, 212, 167, 26, 77, 234, 103, 145, 53, 191, 184, 25, 240, 191, 156, 25,
                60, 144, 65, 22, 193, 163, 246, 227, 212, 81, 49, 170, 33, 158
            ]
        );

        assert_eq!(
            root_keys.cpk.value(),
            &[
                220, 170, 95, 177, 121, 37, 86, 166, 56, 238, 232, 72, 21, 106, 107, 217, 158, 74,
                133, 91, 143, 244, 155, 15, 2, 230, 223, 169, 13, 20, 163, 138
            ]
        );
        assert_eq!(
            child_keys.cpk.value(),
            &[
                152, 249, 236, 111, 132, 96, 184, 122, 21, 179, 240, 15, 234, 155, 164, 144, 108,
                110, 120, 74, 176, 147, 196, 168, 243, 186, 203, 79, 97, 17, 194, 52
            ]
        );
    }

    fn seed_holder_for_tests() -> SeedHolder {
        SeedHolder {
            seed: [42; 64].to_vec(),
        }
    }

    #[test]
    fn test_simple_key_tree() {
        let seed_holder = seed_holder_for_tests();

        let tree = KeyTreePublic::new(&seed_holder);

        assert!(tree.key_map.contains_key(&ChainIndex::root()));
        assert!(tree.addr_map.contains_key(&Address::new([
            46, 223, 229, 177, 59, 18, 189, 219, 153, 31, 249, 90, 112, 230, 180, 164, 80, 25, 106,
            159, 14, 238, 1, 192, 91, 8, 210, 165, 199, 41, 60, 104,
        ])));
    }

    #[test]
    fn test_small_key_tree() {
        let seed_holder = seed_holder_for_tests();

        let mut tree = KeyTreePublic::new(&seed_holder);

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 0);

        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("00000000").unwrap())
        );

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 1);

        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();
        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();
        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();
        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();
        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();
        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 7);
    }

    #[test]
    fn test_key_tree_can_not_make_child_keys() {
        let seed_holder = seed_holder_for_tests();

        let mut tree = KeyTreePublic::new(&seed_holder);

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 0);

        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("00000000").unwrap())
        );

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 1);

        let key_opt = tree.generate_new_pub_keys(ChainIndex::from_str("03000000").unwrap());

        assert_eq!(key_opt, None);
    }

    #[test]
    fn test_key_tree_complex_structure() {
        let seed_holder = seed_holder_for_tests();

        let mut tree = KeyTreePublic::new(&seed_holder);

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 0);

        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("00000000").unwrap())
        );

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 1);

        tree.generate_new_pub_keys(ChainIndex::root()).unwrap();

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("01000000").unwrap())
        );

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::root())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 2);

        tree.generate_new_pub_keys(ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 1);

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("0000000000000000").unwrap())
        );

        tree.generate_new_pub_keys(ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 2);

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("0000000001000000").unwrap())
        );

        tree.generate_new_pub_keys(ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::from_str("00000000").unwrap())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 3);

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("0000000002000000").unwrap())
        );

        tree.generate_new_pub_keys(ChainIndex::from_str("0000000001000000").unwrap())
            .unwrap();

        assert!(
            tree.key_map
                .contains_key(&ChainIndex::from_str("000000000100000000000000").unwrap())
        );

        let next_last_child_for_parent_id = tree
            .find_next_last_child_of_id(&ChainIndex::from_str("0000000001000000").unwrap())
            .unwrap();

        assert_eq!(next_last_child_for_parent_id, 1);
    }
}
