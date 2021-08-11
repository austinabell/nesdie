//* NO_STD not possible because miniserde requires it (for now)
#![cfg_attr(target_arch = "wasm32", no_std)]

mod utils;

#[cfg(target_arch = "wasm32")]
mod no_std_contract;

extern crate alloc;

use alloc::boxed::Box;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use borsh::{self, BorshDeserialize, BorshSerialize};
use miniserde::__private::Cow;
use miniserde::de::Visitor;
use miniserde::ser::Fragment;
use miniserde::{make_place, Deserialize, Serialize};
use nesdie::env;
use nesdie_collections::legacy_unordered_map::UnorderedMap;
use utils::json::{Base64VecU8, U128, U64};
use utils::types::{AccountId, Promise, PromiseOrValue, PublicKey};

use crate::utils::helper_env;

/// Unlimited allowance for multisig keys.
const DEFAULT_ALLOWANCE: u128 = 0;

// Request cooldown period (time before a request can be deleted)
const REQUEST_COOLDOWN: u64 = 900_000_000_000;

pub type RequestId = u32;

/// Permissions for function call access key.
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct FunctionCallPermission {
    allowance: Option<U128>,
    receiver_id: AccountId,
    method_names: Vec<String>,
}

/// Lowest level action that can be performed by the multisig contract.
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum MultiSigRequestAction {
    /// Transfers given amount to receiver.
    Transfer { amount: U128 },
    /// Create a new account.
    CreateAccount,
    /// Deploys contract to receiver's account. Can upgrade given contract as well.
    DeployContract { code: Base64VecU8 },
    /// Adds key, either new key for multisig or full access key to another account.
    AddKey {
        public_key: PublicKey,
        permission: Option<FunctionCallPermission>,
    },
    /// Deletes key, either one of the keys from multisig or key from another account.
    DeleteKey { public_key: PublicKey },
    /// Call function on behalf of this contract.
    FunctionCall {
        method_name: String,
        args: Base64VecU8,
        deposit: U128,
        gas: U64,
    },
    /// Sets number of confirmations required to authorize requests.
    /// Can not be bundled with any other actions or transactions.
    SetNumConfirmations { num_confirmations: u32 },
    /// Sets number of active requests (unconfirmed requests) per access key
    /// Default is 12 unconfirmed requests at a time
    /// The REQUEST_COOLDOWN for requests is 15min
    /// Worst gas attack a malicious keyholder could do is 12 requests every 15min
    SetActiveRequestsLimit { active_requests_limit: u32 },
}

struct MultiSigRequestActionStream<'a> {
    data: &'a MultiSigRequestAction,
    state: usize,
}

impl<'a> miniserde::ser::Map for MultiSigRequestActionStream<'a> {
    fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
        let state = self.state;
        self.state += 1;
        use self::MultiSigRequestAction as Msa;
        match self.data {
            Msa::Transfer { amount } => match state {
                0 => Some((Cow::Borrowed("type"), &"Transfer")),
                1 => Some((Cow::Borrowed("amount"), amount)),
                _ => None,
            },
            Msa::CreateAccount => match state {
                0 => Some((Cow::Borrowed("type"), &"CreateAccount")),
                _ => None,
            },
            Msa::DeployContract { code } => match state {
                0 => Some((Cow::Borrowed("type"), &"DeployContract")),
                1 => Some((Cow::Borrowed("code"), code)),
                _ => None,
            },
            Msa::AddKey {
                public_key,
                permission,
            } => match state {
                0 => Some((Cow::Borrowed("type"), &"AddKey")),
                1 => Some((Cow::Borrowed("public_key"), public_key)),
                2 => Some((Cow::Borrowed("permission"), permission)),
                _ => None,
            },
            Msa::DeleteKey { public_key } => match state {
                0 => Some((Cow::Borrowed("type"), &"DeleteKey")),
                1 => Some((Cow::Borrowed("public_key"), public_key)),
                _ => None,
            },
            Msa::FunctionCall {
                method_name,
                args,
                deposit,
                gas,
            } => match state {
                0 => Some((Cow::Borrowed("type"), &"FunctionCall")),
                1 => Some((Cow::Borrowed("method_name"), method_name)),
                2 => Some((Cow::Borrowed("args"), args)),
                3 => Some((Cow::Borrowed("deposit"), deposit)),
                4 => Some((Cow::Borrowed("gas"), gas)),
                _ => None,
            },
            Msa::SetNumConfirmations { num_confirmations } => match state {
                0 => Some((Cow::Borrowed("type"), &"SetNumConfirmations")),
                1 => Some((Cow::Borrowed("num_confirmations"), num_confirmations)),
                _ => None,
            },
            Msa::SetActiveRequestsLimit {
                active_requests_limit,
            } => match state {
                0 => Some((Cow::Borrowed("type"), &"SetActiveRequestsLimit")),
                1 => Some((
                    Cow::Borrowed("active_requests_limit"),
                    active_requests_limit,
                )),
                _ => None,
            },
        }
    }
}

impl miniserde::Serialize for MultiSigRequestAction {
    fn begin(&self) -> Fragment {
        Fragment::Map(Box::new(MultiSigRequestActionStream {
            data: self,
            state: 0,
        }))
    }
}

make_place!(Place);
impl Visitor for Place<MultiSigRequestAction> {
    fn map(&mut self) -> miniserde::Result<Box<dyn miniserde::de::Map + '_>> {
        // Like for sequences, we produce a builder that can hand out places
        // to write one struct field at a time.
        Ok(Box::new(MsraBuilder {
            ty: None,
            amount: None,
            code: None,
            public_key: None,
            permission: None,
            delete_key: None,
            method_name: None,
            args: None,
            deposit: None,
            gas: None,
            num_confirmations: None,
            active_requests_limit: None,
            out: &mut self.out,
        }))
    }
}

struct MsraBuilder<'a> {
    ty: Option<String>,
    amount: Option<U128>,
    code: Option<Base64VecU8>,
    public_key: Option<PublicKey>,
    permission: Option<FunctionCallPermission>,
    delete_key: Option<PublicKey>,
    method_name: Option<String>,
    args: Option<Base64VecU8>,
    deposit: Option<U128>,
    gas: Option<U64>,
    num_confirmations: Option<u32>,
    active_requests_limit: Option<u32>,
    out: &'a mut Option<MultiSigRequestAction>,
}

#[allow(bare_trait_objects)]
impl<'a> miniserde::de::Map for MsraBuilder<'a> {
    fn key(&mut self, k: &str) -> miniserde::Result<&mut dyn Visitor> {
        match k {
            "type" => Ok(Deserialize::begin(&mut self.ty)),
            "code" => Ok(Deserialize::begin(&mut self.code)),
            "amount" => Ok(Deserialize::begin(&mut self.amount)),
            "public_key" => Ok(Deserialize::begin(&mut self.public_key)),
            "permission" => Ok(Deserialize::begin(&mut self.permission)),
            "delete_key" => Ok(Deserialize::begin(&mut self.delete_key)),
            "method_name" => Ok(Deserialize::begin(&mut self.method_name)),
            "args" => Ok(Deserialize::begin(&mut self.args)),
            "deposit" => Ok(Deserialize::begin(&mut self.deposit)),
            "gas" => Ok(Deserialize::begin(&mut self.gas)),
            "num_confirmations" => Ok(Deserialize::begin(&mut self.num_confirmations)),
            "active_requests_limit" => Ok(Deserialize::begin(&mut self.active_requests_limit)),
            _ => Ok(Visitor::ignore()),
        }
    }

    fn finish(&mut self) -> miniserde::Result<()> {
        match self.ty.as_ref().ok_or(miniserde::Error)?.as_str() {
            "Transfer" => {
                let amount = self.amount.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::Transfer { amount });
            }
            "CreateAccount" => {
                *self.out = Some(MultiSigRequestAction::CreateAccount);
            }
            "DeployContract" => {
                let code = self.code.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::DeployContract { code });
            }
            "AddKey" => {
                let public_key = self.public_key.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::AddKey {
                    public_key,
                    permission: core::mem::take(&mut self.permission),
                });
            }
            "DeleteKey" => {
                let public_key = self.public_key.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::DeleteKey { public_key });
            }
            "FunctionCall" => {
                let method_name = self.method_name.take().ok_or(miniserde::Error)?;
                let args = self.args.take().ok_or(miniserde::Error)?;
                let deposit = self.deposit.take().ok_or(miniserde::Error)?;
                let gas = self.gas.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::FunctionCall {
                    method_name,
                    args,
                    deposit,
                    gas,
                });
            }
            "SetNumConfirmations" => {
                let num_confirmations = self.num_confirmations.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::SetNumConfirmations { num_confirmations });
            }
            "SetActiveRequestsLimit" => {
                let active_requests_limit =
                    self.active_requests_limit.take().ok_or(miniserde::Error)?;
                *self.out = Some(MultiSigRequestAction::SetActiveRequestsLimit {
                    active_requests_limit,
                });
            }
            _ => return Err(miniserde::Error),
        }
        Ok(())
    }
}

impl miniserde::Deserialize for MultiSigRequestAction {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

// The request the user makes specifying the receiving account and actions they want to execute (1 tx)
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct MultiSigRequest {
    receiver_id: AccountId,
    actions: Vec<MultiSigRequestAction>,
}

// An internal request wrapped with the signer_pk and added timestamp to determine num_requests_pk and prevent against malicious key holder gas attacks
#[derive(Clone, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct MultiSigRequestWithSigner {
    request: MultiSigRequest,
    signer_pk: PublicKey,
    added_timestamp: u64,
}

// #[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MultiSigContract {
    num_confirmations: u32,
    request_nonce: RequestId,
    requests: UnorderedMap<RequestId, MultiSigRequestWithSigner>,
    confirmations: UnorderedMap<RequestId, BTreeSet<PublicKey>>,
    num_requests_pk: UnorderedMap<PublicKey, u32>,
    // per key
    active_requests_limit: u32,
}

// If you haven't initialized the contract with new(num_confirmations: u32)
impl Default for MultiSigContract {
    fn default() -> Self {
        env::panic_str("Multisig contract should be initialized before usage")
    }
}

// #[near_bindgen]
impl MultiSigContract {
    /// Initialize multisig contract.
    /// @params num_confirmations: k of n signatures required to perform operations.
    // #[init]
    pub fn new(num_confirmations: u32) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            num_confirmations,
            request_nonce: 0,
            requests: UnorderedMap::new(b"r".to_vec().into_boxed_slice()),
            confirmations: UnorderedMap::new(b"c".to_vec().into_boxed_slice()),
            num_requests_pk: UnorderedMap::new(b"k".to_vec().into_boxed_slice()),
            active_requests_limit: 12,
        }
    }

    /// Add request for multisig.
    pub fn add_request(&mut self, request: MultiSigRequest) -> RequestId {
        if helper_env::current_account_id() != helper_env::predecessor_account_id() {
            env::panic_str("Predecessor account must much current account")
        }
        // track how many requests this key has made
        let num_requests = self
            .num_requests_pk
            .get(&helper_env::signer_account_pk())
            .copied()
            .unwrap_or(0)
            + 1;
        assert!(
            num_requests <= self.active_requests_limit,
            "Account has too many active requests. Confirm or delete some."
        );
        self.num_requests_pk
            .insert(helper_env::signer_account_pk(), num_requests);
        // add the request
        let request_added = MultiSigRequestWithSigner {
            signer_pk: helper_env::signer_account_pk(),
            added_timestamp: env::block_timestamp(),
            request,
        };
        self.requests.insert(self.request_nonce, request_added);
        let confirmations = BTreeSet::new();
        self.confirmations.insert(self.request_nonce, confirmations);
        self.request_nonce += 1;
        self.request_nonce - 1
    }

    /// Add request for multisig and confirm with the pk that added.
    pub fn add_request_and_confirm(&mut self, request: MultiSigRequest) -> RequestId {
        let request_id = self.add_request(request);
        self.confirm(request_id);
        request_id
    }

    /// Remove given request and associated confirmations.
    pub fn delete_request(&mut self, request_id: RequestId) {
        self.assert_valid_request(request_id);
        let request_with_signer = self.requests.get(&request_id).expect("No such request");
        // can't delete requests before 15min
        assert!(
            env::block_timestamp() > request_with_signer.added_timestamp + REQUEST_COOLDOWN,
            "Request cannot be deleted immediately after creation."
        );
        self.remove_request(request_id);
    }

    fn execute_request(&mut self, request: MultiSigRequest) -> PromiseOrValue<bool> {
        let mut promise = Promise::new(request.receiver_id.clone());
        let receiver_id = request.receiver_id.clone();
        let num_actions = request.actions.len();
        for action in request.actions {
            promise = match action {
                MultiSigRequestAction::Transfer { amount } => promise.transfer(amount.into()),
                MultiSigRequestAction::CreateAccount => promise.create_account(),
                MultiSigRequestAction::DeployContract { code } => {
                    promise.deploy_contract(code.into())
                }
                MultiSigRequestAction::AddKey {
                    public_key,
                    permission,
                } => {
                    self.assert_self_request(receiver_id.clone());
                    if let Some(permission) = permission {
                        promise.add_access_key(
                            public_key,
                            permission
                                .allowance
                                .map(|x| x.into())
                                .unwrap_or(DEFAULT_ALLOWANCE),
                            permission.receiver_id,
                            permission.method_names.join(",").into_bytes(),
                        )
                    } else {
                        // wallet UI should warn user if receiver_id == env::current_account_id(), adding FAK will render multisig useless
                        promise.add_full_access_key(public_key)
                    }
                }
                MultiSigRequestAction::DeleteKey { public_key } => {
                    self.assert_self_request(receiver_id.clone());
                    let pk: PublicKey = public_key;
                    // delete outstanding requests by public_key
                    let request_ids: Vec<u32> = self
                        .requests
                        .iter()
                        .filter(|(_k, r)| r.signer_pk == pk)
                        .map(|(k, _r)| *k)
                        .collect();
                    for request_id in request_ids {
                        // remove confirmations for this request
                        self.confirmations.remove(&request_id);
                        self.requests.remove(&request_id);
                    }
                    // remove num_requests_pk entry for public_key
                    self.num_requests_pk.remove(&pk);
                    promise.delete_key(pk)
                }
                MultiSigRequestAction::FunctionCall {
                    method_name,
                    args,
                    deposit,
                    gas,
                } => promise.function_call(
                    method_name.into_bytes(),
                    args.into(),
                    deposit.into(),
                    gas.into(),
                ),
                // the following methods must be a single action
                MultiSigRequestAction::SetNumConfirmations { num_confirmations } => {
                    self.assert_one_action_only(receiver_id, num_actions);
                    self.num_confirmations = num_confirmations;
                    return PromiseOrValue::Value(true);
                }
                MultiSigRequestAction::SetActiveRequestsLimit {
                    active_requests_limit,
                } => {
                    self.assert_one_action_only(receiver_id, num_actions);
                    self.active_requests_limit = active_requests_limit;
                    return PromiseOrValue::Value(true);
                }
            };
        }
        promise.into()
    }

    /// Confirm given request with given signing key.
    /// If with this, there has been enough confirmation, a promise with request will be scheduled.
    pub fn confirm(&mut self, request_id: RequestId) -> PromiseOrValue<bool> {
        self.assert_valid_request(request_id);
        let mut confirmations = self
            .confirmations
            .get(&request_id)
            .cloned()
            .unwrap_or_else(|| unreachable!());
        assert!(
            !confirmations.contains(&helper_env::signer_account_pk()),
            "Already confirmed this request with this key"
        );
        if confirmations.len() as u32 + 1 >= self.num_confirmations {
            let request = self.remove_request(request_id);
            /********************************
            NOTE: If the tx execution fails for any reason, the request and confirmations are removed already, so the client has to start all over
            ********************************/
            self.execute_request(request)
        } else {
            confirmations.insert(helper_env::signer_account_pk());
            self.confirmations.insert(request_id, confirmations.clone());
            PromiseOrValue::Value(true)
        }
    }

    /********************************
    Helper methods
    ********************************/
    // removes request, removes confirmations and reduces num_requests_pk - used in delete, delete_key, and confirm
    fn remove_request(&mut self, request_id: RequestId) -> MultiSigRequest {
        // remove confirmations for this request
        self.confirmations.remove(&request_id);
        // remove the original request
        let request_with_signer = self
            .requests
            .remove(&request_id)
            .expect("Failed to remove existing element");
        // decrement num_requests for original request signer
        let original_signer_pk = request_with_signer.signer_pk;
        let mut num_requests = self
            .num_requests_pk
            .get(&original_signer_pk)
            .copied()
            .unwrap_or(0);
        // safety check for underrun (unlikely since original_signer_pk must have num_requests_pk > 0)
        if num_requests > 0 {
            num_requests -= 1;
        }
        self.num_requests_pk
            .insert(original_signer_pk, num_requests);
        // return request
        request_with_signer.request
    }
    // Prevents access to calling requests and make sure request_id is valid - used in delete and confirm
    fn assert_valid_request(&mut self, request_id: RequestId) {
        // request must come from key added to contract account
        assert_eq!(
            helper_env::current_account_id(),
            helper_env::predecessor_account_id(),
            "Predecessor account must much current account"
        );
        // request must exist
        assert!(
            self.requests.get(&request_id).is_some(),
            "No such request: either wrong number or already confirmed"
        );
        // request must have
        assert!(
            self.confirmations.get(&request_id).is_some(),
            "Internal error: confirmations mismatch requests"
        );
    }
    // Prevents request from approving tx on another account
    fn assert_self_request(&mut self, receiver_id: AccountId) {
        assert_eq!(
            receiver_id,
            helper_env::current_account_id(),
            "This method only works when receiver_id is equal to current_account_id"
        );
    }
    // Prevents a request from being bundled with other actions
    fn assert_one_action_only(&mut self, receiver_id: AccountId, num_actions: usize) {
        self.assert_self_request(receiver_id);
        assert_eq!(num_actions, 1, "This method should be a separate request");
    }
    /********************************
    View methods
    ********************************/
    pub fn get_request(&self, request_id: RequestId) -> MultiSigRequest {
        (self
            .requests
            .get(&request_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("No such request")))
        .request
    }

    pub fn get_num_requests_pk(&self, public_key: PublicKey) -> u32 {
        self.num_requests_pk.get(&public_key).copied().unwrap_or(0)
    }

    pub fn list_request_ids(&self) -> Vec<RequestId> {
        self.requests.keys().copied().collect()
    }

    pub fn get_confirmations(&self, request_id: RequestId) -> Vec<PublicKey> {
        self.confirmations
            .get(&request_id)
            .map(|m| m.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_num_confirmations(&self) -> u32 {
        self.num_confirmations
    }

    pub fn get_request_nonce(&self) -> u32 {
        self.request_nonce
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Error, Formatter};

    use nesdie::mock::{VMContext, VmContextBuilder};
    use nesdie::testing_env;
    use nesdie::Balance;
    use utils::types::AccountId;

    use super::*;

    /// Used for asserts_eq.
    /// TODO: replace with derive when https://github.com/near/near-sdk-rs/issues/165
    impl Debug for MultiSigRequest {
        fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), Error> {
            panic!("Should not trigger");
        }
    }

    pub fn alice() -> AccountId {
        "alice".to_string()
    }
    pub fn bob() -> AccountId {
        "bob".to_string()
    }

    fn context_with_key(key: PublicKey, amount: Balance) -> VMContext {
        VmContextBuilder::new()
            .current_account_id(alice())
            .predecessor_account_id(alice())
            .signer_account_id(alice())
            .signer_account_pk(key.into())
            .account_balance(amount)
            .build()
    }

    fn context_with_key_future(key: PublicKey, amount: Balance) -> VMContext {
        VmContextBuilder::new()
            .current_account_id(alice())
            .block_timestamp(REQUEST_COOLDOWN + 1)
            .predecessor_account_id(alice())
            .signer_account_id(alice())
            .signer_account_pk(key.into())
            .account_balance(amount)
            .build()
    }

    fn dummy_public_key() -> PublicKey {
        "Eg2jtsiMrprn7zgaKUk79qM1hWhANsFyE6JSX4txLEub"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_multi_3_of_n() {
        let amount = 1_000;
        testing_env!(context_with_key(
            "Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy"
                .parse()
                .unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        };
        let request_id = c.add_request(request.clone());
        assert_eq!(c.get_request(request_id), request);
        assert_eq!(c.list_request_ids(), vec![request_id]);
        c.confirm(request_id);
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        testing_env!(context_with_key(
            "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
                .parse()
                .unwrap(),
            amount
        ));
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 2);
        assert_eq!(c.get_confirmations(request_id).len(), 2);
        testing_env!(context_with_key(
            "2EfbwnQHPBWQKbNczLiVznFghh9qs716QT71zN6L1D95"
                .parse()
                .unwrap(),
            amount
        ));
        c.confirm(request_id);
        // TODO: confirm that funds were transferred out via promise.
        assert_eq!(c.requests.len(), 0);
    }

    #[test]
    fn test_multi_add_request_and_confirm() {
        let amount = 1_000;
        testing_env!(context_with_key(
            "Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy"
                .parse()
                .unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(3);
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        };
        let request_id = c.add_request_and_confirm(request.clone());
        assert_eq!(c.get_request(request_id), request);
        assert_eq!(c.list_request_ids(), vec![request_id]);
        // c.confirm(request_id);
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        testing_env!(context_with_key(
            "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
                .parse()
                .unwrap(),
            amount
        ));
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 2);
        assert_eq!(c.get_confirmations(request_id).len(), 2);
        testing_env!(context_with_key(
            "2EfbwnQHPBWQKbNczLiVznFghh9qs716QT71zN6L1D95"
                .parse()
                .unwrap(),
            amount
        ));
        c.confirm(request_id);
        // TODO: confirm that funds were transferred out via promise.
        assert_eq!(c.requests.len(), 0);
    }

    #[test]
    fn add_key_delete_key_storage_cleared() {
        let amount = 1_000;
        testing_env!(context_with_key(
            "Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy"
                .parse()
                .unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(1);
        let new_key: PublicKey = "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
            .parse()
            .unwrap();
        // vm current_account_id is alice, receiver_id must be alice
        let request = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::AddKey {
                public_key: new_key.clone(),
                permission: None,
            }],
        };
        // make request
        c.add_request_and_confirm(request.clone());
        // should be empty now
        assert_eq!(c.requests.len(), 0);
        // switch accounts
        testing_env!(context_with_key(
            "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
                .parse()
                .unwrap(),
            amount
        ));
        let request2 = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        };
        // make request but don't confirm
        c.add_request(request2.clone());
        // should have 1 request now
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.get_num_requests_pk(new_key.clone()), 1);
        // self delete key
        let request3 = MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::DeleteKey {
                public_key: new_key.clone(),
            }],
        };
        // make request and confirm
        c.add_request_and_confirm(request3.clone());
        // should be empty now
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.get_num_requests_pk(new_key.clone()), 0);
    }

    #[test]
    #[should_panic]
    fn test_panics_add_key_different_account() {
        let amount = 1_000;
        testing_env!(context_with_key(
            "Eg2jtsiMrprn7zgKKUk79qM1hWhANsFyE6JSX4txLEuy"
                .parse()
                .unwrap(),
            amount
        ));
        let mut c = MultiSigContract::new(1);
        let new_key: PublicKey = "HghiythFFPjVXwc9BLNi8uqFmfQc1DWFrJQ4nE6ANo7R"
            .parse()
            .unwrap();
        // vm current_account_id is alice, receiver_id must be alice
        let request = MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::AddKey {
                public_key: new_key.clone(),
                permission: None,
            }],
        };
        // make request
        c.add_request_and_confirm(request);
    }

    #[test]
    fn test_change_num_confirmations() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(1);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: alice(),
            actions: vec![MultiSigRequestAction::SetNumConfirmations {
                num_confirmations: 2,
            }],
        });
        c.confirm(request_id);
        assert_eq!(c.num_confirmations, 2);
    }

    #[test]
    #[should_panic]
    fn test_panics_on_second_confirm() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        });
        assert_eq!(c.requests.len(), 1);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 0);
        c.confirm(request_id);
        assert_eq!(c.confirmations.get(&request_id).unwrap().len(), 1);
        c.confirm(request_id);
    }

    #[test]
    #[should_panic]
    fn test_panics_delete_request() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        });
        c.delete_request(request_id);
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.confirmations.len(), 0);
    }

    #[test]
    fn test_delete_request_future() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        });
        testing_env!(context_with_key_future(dummy_public_key(), amount));
        c.delete_request(request_id);
        assert_eq!(c.requests.len(), 0);
        assert_eq!(c.confirmations.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_delete_request_panic_wrong_key() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(3);
        let request_id = c.add_request(MultiSigRequest {
            receiver_id: bob(),
            actions: vec![MultiSigRequestAction::Transfer {
                amount: amount.into(),
            }],
        });
        testing_env!(context_with_key(dummy_public_key(), amount));
        c.delete_request(request_id);
    }

    #[test]
    #[should_panic]
    fn test_too_many_requests() {
        let amount = 1_000;
        testing_env!(context_with_key(dummy_public_key(), amount));
        let mut c = MultiSigContract::new(3);
        for _i in 0..16 {
            c.add_request(MultiSigRequest {
                receiver_id: bob(),
                actions: vec![MultiSigRequestAction::Transfer {
                    amount: amount.into(),
                }],
            });
        }
    }
}
