extern crate alloc;

use crate::{Balance, Gas};
use alloc::string::String;
use alloc::vec::Vec;

use near_vm_logic::VMContext;

type AccountId = String;
type PublicKey = Vec<u8>;
type BlockHeight = u64;
type EpochHeight = u64;

/// Simple VMContext builder that allows to quickly create custom context in tests.
#[derive(Clone)]
pub struct VmContextBuilder {
    context: VMContext,
}

impl Default for VmContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(missing_docs)]
impl VmContextBuilder {
    pub fn new() -> Self {
        Self {
            context: VMContext {
                current_account_id: "alice".into(),
                signer_account_id: "bob".into(),
                signer_account_pk: vec![0u8; 32],
                predecessor_account_id: "bob".into(),
                input: vec![],
                block_index: 0,
                block_timestamp: 0,
                epoch_height: 0,
                account_balance: 10u128.pow(26),
                account_locked_balance: 0,
                storage_usage: 1024 * 300,
                attached_deposit: 0,
                prepaid_gas: 300 * 10u64.pow(12),
                random_seed: vec![0u8; 32],
                is_view: false,
                output_data_receivers: vec![],
            },
        }
    }

    pub fn current_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.current_account_id = account_id;
        self
    }

    pub fn signer_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.signer_account_id = account_id;
        self
    }

    pub fn signer_account_pk(&mut self, pk: PublicKey) -> &mut Self {
        self.context.signer_account_pk = pk;
        self
    }

    pub fn predecessor_account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.context.predecessor_account_id = account_id;
        self
    }

    pub fn block_index(&mut self, block_index: BlockHeight) -> &mut Self {
        self.context.block_index = block_index;
        self
    }

    pub fn block_timestamp(&mut self, block_timestamp: u64) -> &mut Self {
        self.context.block_timestamp = block_timestamp;
        self
    }

    pub fn epoch_height(&mut self, epoch_height: EpochHeight) -> &mut Self {
        self.context.epoch_height = epoch_height;
        self
    }

    pub fn account_balance(&mut self, amount: Balance) -> &mut Self {
        self.context.account_balance = amount;
        self
    }

    pub fn account_locked_balance(&mut self, amount: Balance) -> &mut Self {
        self.context.account_locked_balance = amount;
        self
    }

    pub fn storage_usage(&mut self, usage: u64) -> &mut Self {
        self.context.storage_usage = usage;
        self
    }

    pub fn attached_deposit(&mut self, amount: Balance) -> &mut Self {
        self.context.attached_deposit = amount;
        self
    }

    pub fn prepaid_gas(&mut self, gas: Gas) -> &mut Self {
        self.context.prepaid_gas = gas;
        self
    }

    pub fn random_seed(&mut self, seed: Vec<u8>) -> &mut Self {
        self.context.random_seed = seed;
        self
    }

    pub fn is_view(&mut self, is_view: bool) -> &mut Self {
        self.context.is_view = is_view;
        self
    }

    pub fn build(&self) -> VMContext {
        self.context.clone()
    }
}
