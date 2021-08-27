#![allow(missing_docs)]

use crate::{Balance, Gas};

type PublicKey = Vec<u8>;
type AccountId = String;

#[derive(Clone, Debug)]
pub struct Receipt {
    pub receipt_indices: Vec<u64>,
    pub receiver_id: String,
    pub actions: Vec<VmAction>,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum VmAction {
    CreateAccount,
    DeployContract {
        code: Vec<u8>,
    },
    FunctionCall {
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    },
    Transfer {
        deposit: Balance,
    },
    Stake {
        stake: Balance,
        public_key: PublicKey,
    },
    AddKeyWithFullAccess {
        public_key: PublicKey,
        nonce: u64,
    },
    AddKeyWithFunctionCall {
        public_key: PublicKey,
        nonce: u64,
        allowance: Option<Balance>,
        receiver_id: AccountId,
        method_names: Vec<String>,
    },
    DeleteKey {
        public_key: PublicKey,
    },
    DeleteAccount {
        beneficiary_id: AccountId,
    },
}
