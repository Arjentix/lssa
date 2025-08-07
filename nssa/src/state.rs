use crate::{
    AuthenticatedTransferProgram, address::Address, execute_public,
    public_transaction::PublicTransaction,
};
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{Program, validate_constraints},
};
use std::collections::{HashMap, HashSet};

struct V01State {
    public_state: HashMap<Address, Account>,
}

impl V01State {
    fn transition_from_public_transaction(&mut self, tx: PublicTransaction) -> Result<(), ()> {
        let state_diff = self
            .execute_and_verify_public_transaction(&tx)
            .map_err(|_| ())?;

        for (address, post) in state_diff.into_iter() {
            let current_account = self.get_account_by_address_mut(address);
            *current_account = post;
        }

        for address in tx.signer_addresses() {
            let current_account = self.get_account_by_address_mut(address);
            current_account.nonce += 1;
        }
        Ok(())
    }

    fn get_account_by_address_mut(&mut self, address: Address) -> &mut Account {
        self.public_state
            .entry(address)
            .or_insert_with(Account::default)
    }

    fn get_account_by_address(&self, address: &Address) -> Account {
        self.public_state
            .get(address)
            .cloned()
            .unwrap_or(Account::default())
    }

    fn execute_and_verify_public_transaction(
        &mut self,
        tx: &PublicTransaction,
    ) -> Result<HashMap<Address, Account>, ()> {
        let message = tx.message();
        let witness_set = tx.witness_set();

        // All addresses must be different
        if message.addresses.iter().collect::<HashSet<_>>().len() != message.addresses.len() {
            return Err(());
        }

        if message.nonces.len() != witness_set.signatures_and_public_keys.len() {
            return Err(());
        }

        let mut authorized_addresses = Vec::new();
        for ((signature, public_key), nonce) in witness_set
            .signatures_and_public_keys
            .iter()
            .zip(message.nonces.iter())
        {
            // Check the signature is valid
            if !signature.is_valid_for(message, public_key) {
                return Err(());
            }

            // Check the nonce corresponds to the current nonce on the public state.
            let address = Address::from_public_key(public_key);
            let current_nonce = self.get_account_by_address(&address).nonce;
            if current_nonce != *nonce {
                return Err(());
            }

            authorized_addresses.push(address);
        }

        // Build pre_states for execution
        let pre_states: Vec<_> = message
            .addresses
            .iter()
            .map(|address| AccountWithMetadata {
                account: self.get_account_by_address(address),
                is_authorized: authorized_addresses.contains(address),
            })
            .collect();

        // Check the `program_id` corresponds to a built-in program
        // Only allowed program so far is the authenticated transfer program
        if message.program_id != AuthenticatedTransferProgram::PROGRAM_ID {
            return Err(());
        }

        // // Execute program
        let post_states =
            execute_public::<AuthenticatedTransferProgram>(&pre_states, message.instruction_data)
                .map_err(|_| ())?;

        // Verify execution corresponds to a well-behaved program.
        // See the # Programs section for the definition of the `validate_constraints` method.
        validate_constraints(&pre_states, &post_states, message.program_id).map_err(|_| ())?;

        if (post_states.len() != message.addresses.len()) {
            return Err(());
        }

        Ok(message
            .addresses
            .iter()
            .cloned()
            .zip(post_states.into_iter())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{public_transaction, signature::PrivateKey};

    fn genesis_state_for_tests(balances: &[u128], addresses: &[Address]) -> V01State {
        assert_eq!(balances.len(), addresses.len());
        let accounts: Vec<Account> = balances
            .iter()
            .map(|balance| {
                let mut account = Account::default();
                account.balance = *balance;
                account.program_owner = AuthenticatedTransferProgram::PROGRAM_ID;
                account
            })
            .collect();

        let public_state = addresses
            .to_owned()
            .into_iter()
            .zip(accounts.into_iter())
            .collect();
        V01State { public_state }
    }

    fn transfer_transaction_for_tests(
        from: Address,
        from_key: PrivateKey,
        nonce: u128,
        to: Address,
        balance: u128,
    ) -> PublicTransaction {
        let addresses = vec![from, to];
        let nonces = vec![nonce];
        let program_id = AuthenticatedTransferProgram::PROGRAM_ID;
        let message = public_transaction::Message::new(program_id, addresses, nonces, balance);
        let witness_set = public_transaction::WitnessSet::for_message(&message, &[from_key]);
        PublicTransaction::new(message, witness_set)
    }

    #[test]
    fn test_1() {
        let addresses = [Address::new([1; 32])];
        let balances = [100];
        let mut genesis_state = genesis_state_for_tests(&balances, &addresses);
        let from = addresses[0].clone();
        let from_key = PrivateKey(1);
        let to = Address::new([2; 32]);
        let balance_to_move = 5;
        let tx =
            transfer_transaction_for_tests(from.clone(), from_key, 0, to.clone(), balance_to_move);
        let _ = genesis_state.transition_from_public_transaction(tx);
        assert_eq!(
            genesis_state.get_account_by_address(&to).balance,
            balance_to_move
        );
        assert_eq!(
            genesis_state.get_account_by_address(&from).balance,
            balances[0] - balance_to_move
        );
        assert_eq!(genesis_state.get_account_by_address(&from).nonce, 1);
        assert_eq!(genesis_state.get_account_by_address(&to).nonce, 0);
    }

    #[test]
    fn test_2() {
        let addresses = [Address::new([1; 32]), Address::new([99; 32])];
        let balances = [100, 200];
        let mut genesis_state = genesis_state_for_tests(&balances, &addresses);
        let from = addresses[1].clone();
        let from_key = PrivateKey(99);
        let to = addresses[0].clone();
        let balance_to_move = 8;
        let to_previous_balance = genesis_state.get_account_by_address(&to).balance;
        let tx = transfer_transaction_for_tests(from.clone(), from_key, 0, to.clone(), balance_to_move);
        let _ = genesis_state.transition_from_public_transaction(tx);
        assert_eq!(genesis_state.get_account_by_address(&to).balance, 108);
        assert_eq!(
            genesis_state.get_account_by_address(&from).balance,
            balances[1] - balance_to_move
        );
        assert_eq!(genesis_state.get_account_by_address(&from).nonce, 1);
        assert_eq!(genesis_state.get_account_by_address(&to).nonce, 0);
    }
}
