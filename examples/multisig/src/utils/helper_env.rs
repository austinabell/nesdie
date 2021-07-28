#![allow(dead_code)]

use super::types::*;
use super::{public_key::PublicKey, types::AccountId};
use crate::alloc::vec;
use crate::alloc::vec::Vec;
use core::convert::TryFrom;
use core::mem::size_of;
use nesdie::{env, sys};

const ATOMIC_OP_REGISTER: u64 = 0;

/// Register used to record evicted values from the storage.
const EVICTED_REGISTER: u64 = core::u64::MAX - 1;

/// Key used to store the state of the contract.
const STATE_KEY: &[u8] = b"STATE";

/// The minimum length of a valid account ID.
const MIN_ACCOUNT_ID_LEN: u64 = 2;
/// The maximum length of a valid account ID.
const MAX_ACCOUNT_ID_LEN: u64 = 64;

macro_rules! try_method_into_register {
    ( $method:ident ) => {{
        unsafe { sys::$method(ATOMIC_OP_REGISTER) };
        read_register(ATOMIC_OP_REGISTER)
    }};
}

/// Same as `try_method_into_register` but expects the data.
macro_rules! method_into_register {
    ( $method:ident ) => {{
        try_method_into_register!($method).unwrap_or_else(|| unreachable!())
    }};
}

// /// Implements panic hook that converts `PanicInfo` into a string and provides it through the
// /// blockchain interface.
// fn panic_hook_impl(_info: &core::panic::PanicInfo) {
//     //* All errors from this crate should be handled
//     unreachable!()
//     // nesdie::panic_str(info.to_string());
// }

// /// Setups panic hook to expose error info to the blockchain.
// pub fn setup_panic_hook() {
//     std::panic::set_hook(Box::new(panic_hook_impl));
// }

/// The input to the contract call serialized as bytes. If input is not provided returns `None`.
pub fn input() -> Option<Vec<u8>> {
    try_method_into_register!(input)
}

pub fn current_account_id() -> AccountId {
    assert_valid_account_id(method_into_register!(current_account_id))
}

/// The id of the account that either signed the original transaction or issued the initial
/// cross-contract call.
pub fn signer_account_id() -> AccountId {
    assert_valid_account_id(method_into_register!(signer_account_id))
}

/// The public key of the account that did the signing.
pub fn signer_account_pk() -> PublicKey {
    PublicKey::try_from(method_into_register!(signer_account_pk)).unwrap_or_else(|_| unreachable!())
}

/// The id of the account that was the previous contract in the chain of cross-contract calls.
/// If this is the first contract, it is equal to `signer_account_id`.
pub fn predecessor_account_id() -> AccountId {
    assert_valid_account_id(method_into_register!(predecessor_account_id))
}

/// Returns `true` if the given account ID is valid and `false` otherwise.
pub fn is_valid_account_id(account_id: &[u8]) -> bool {
    if (account_id.len() as u64) < MIN_ACCOUNT_ID_LEN
        || (account_id.len() as u64) > MAX_ACCOUNT_ID_LEN
    {
        return false;
    }

    // NOTE: We don't want to use Regex here, because it requires extra time to compile it.
    // The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
    // Instead the implementation is based on the previous character checks.

    // We can safely assume that last char was a separator.
    let mut last_char_is_separator = true;

    for c in account_id {
        let current_char_is_separator = match *c {
            b'a'..=b'z' | b'0'..=b'9' => false,
            b'-' | b'_' | b'.' => true,
            _ => return false,
        };
        if current_char_is_separator && last_char_is_separator {
            return false;
        }
        last_char_is_separator = current_char_is_separator;
    }
    // The account can't end as separator.
    !last_char_is_separator
}

/// Helper function to convert and check the account ID from bytes from the runtime.
fn assert_valid_account_id(bytes: Vec<u8>) -> AccountId {
    if !is_valid_account_id(&bytes) {
        env::panic_str("invalid account id");
    }
    bytes
}

/// Writes key-value into storage.
/// If another key-value existed in the storage with the same key it returns `true`, otherwise `false`.
pub fn storage_write(key: &[u8], value: &[u8]) -> bool {
    match unsafe {
        sys::storage_write(
            key.len() as _,
            key.as_ptr() as _,
            value.len() as _,
            value.as_ptr() as _,
            EVICTED_REGISTER,
        )
    } {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}
/// Reads the value stored under the given key.
pub fn storage_read(key: &[u8]) -> Option<Vec<u8>> {
    match unsafe { sys::storage_read(key.len() as _, key.as_ptr() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => Some(read_register(ATOMIC_OP_REGISTER).unwrap_or_else(|| unreachable!())),
        _ => unreachable!(),
    }
}
/// Removes the value stored under the given key.
/// If key-value existed returns `true`, otherwise `false`.
pub fn storage_remove(key: &[u8]) -> bool {
    match unsafe { sys::storage_remove(key.len() as _, key.as_ptr() as _, EVICTED_REGISTER) } {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}
/// Reads the most recent value that was evicted with `storage_write` or `storage_remove` command.
pub fn storage_get_evicted() -> Option<Vec<u8>> {
    read_register(EVICTED_REGISTER)
}
/// Checks if there is a key-value in the storage.
pub fn storage_has_key(key: &[u8]) -> bool {
    match unsafe { sys::storage_has_key(key.len() as _, key.as_ptr() as _) } {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}

pub fn state_read<T: borsh::BorshDeserialize>() -> Option<T> {
    storage_read(STATE_KEY).map(|data| {
        T::try_from_slice(&data)
            .unwrap_or_else(|_| env::panic_str("Cannot deserialize the contract state."))
    })
}

pub fn state_write<T: borsh::BorshSerialize>(state: &T) {
    let data = state.try_to_vec().unwrap_or_else(|_| unreachable!());
    storage_write(STATE_KEY, &data);
}

/// Returns `true` if the contract state exists and `false` otherwise.
pub fn state_exists() -> bool {
    storage_has_key(STATE_KEY)
}

//* Promises

// Creates a promise that will execute a method on account with given arguments and attaches
/// the given amount and gas.
pub fn promise_create(
    account_id: AccountId,
    method_name: &[u8],
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) -> PromiseIndex {
    let account_id = account_id.as_slice();
    unsafe {
        sys::promise_create(
            account_id.len() as _,
            account_id.as_ptr() as _,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        )
    }
}

/// Attaches the callback that is executed after promise pointed by `promise_idx` is complete.
pub fn promise_then(
    promise_idx: PromiseIndex,
    account_id: AccountId,
    method_name: &[u8],
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) -> PromiseIndex {
    let account_id = account_id.as_slice();
    unsafe {
        sys::promise_then(
            promise_idx,
            account_id.len() as _,
            account_id.as_ptr() as _,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        )
    }
}

/// Creates a new promise which completes when time all promises passed as arguments complete.
pub fn promise_and(promise_indices: &[PromiseIndex]) -> PromiseIndex {
    let mut data = vec![0u8; promise_indices.len() * size_of::<PromiseIndex>()];
    for i in 0..promise_indices.len() {
        data[i * size_of::<PromiseIndex>()..(i + 1) * size_of::<PromiseIndex>()]
            .copy_from_slice(&promise_indices[i].to_le_bytes());
    }
    unsafe { sys::promise_and(data.as_ptr() as _, promise_indices.len() as _) }
}

pub fn promise_batch_create(account_id: &[u8]) -> PromiseIndex {
    unsafe { sys::promise_batch_create(account_id.len() as _, account_id.as_ptr() as _) }
}

pub fn promise_batch_then(promise_index: PromiseIndex, account_id: &[u8]) -> PromiseIndex {
    unsafe {
        sys::promise_batch_then(
            promise_index,
            account_id.len() as _,
            account_id.as_ptr() as _,
        )
    }
}

pub fn promise_batch_action_create_account(promise_index: PromiseIndex) {
    unsafe { sys::promise_batch_action_create_account(promise_index) }
}

pub fn promise_batch_action_deploy_contract(promise_index: u64, code: &[u8]) {
    unsafe {
        sys::promise_batch_action_deploy_contract(
            promise_index,
            code.len() as _,
            code.as_ptr() as _,
        )
    }
}

pub fn promise_batch_action_function_call(
    promise_index: PromiseIndex,
    method_name: &[u8],
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) {
    unsafe {
        sys::promise_batch_action_function_call(
            promise_index,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        )
    }
}

pub fn promise_batch_action_transfer(promise_index: PromiseIndex, amount: Balance) {
    unsafe { sys::promise_batch_action_transfer(promise_index, &amount as *const Balance as _) }
}

pub fn promise_batch_action_stake(
    promise_index: PromiseIndex,
    amount: Balance,
    public_key: &PublicKey,
) {
    unsafe {
        sys::promise_batch_action_stake(
            promise_index,
            &amount as *const Balance as _,
            public_key.as_bytes().len() as _,
            public_key.as_bytes().as_ptr() as _,
        )
    }
}
pub fn promise_batch_action_add_key_with_full_access(
    promise_index: PromiseIndex,
    public_key: &PublicKey,
    nonce: u64,
) {
    unsafe {
        sys::promise_batch_action_add_key_with_full_access(
            promise_index,
            public_key.as_bytes().len() as _,
            public_key.as_bytes().as_ptr() as _,
            nonce,
        )
    }
}
pub fn promise_batch_action_add_key_with_function_call(
    promise_index: PromiseIndex,
    public_key: &PublicKey,
    nonce: u64,
    allowance: Balance,
    receiver_id: &[u8],
    method_names: &[u8],
) {
    unsafe {
        sys::promise_batch_action_add_key_with_function_call(
            promise_index,
            public_key.as_bytes().len() as _,
            public_key.as_bytes().as_ptr() as _,
            nonce,
            &allowance as *const Balance as _,
            receiver_id.len() as _,
            receiver_id.as_ptr() as _,
            method_names.len() as _,
            method_names.as_ptr() as _,
        )
    }
}
pub fn promise_batch_action_delete_key(promise_index: PromiseIndex, public_key: &PublicKey) {
    unsafe {
        sys::promise_batch_action_delete_key(
            promise_index,
            public_key.as_bytes().len() as _,
            public_key.as_bytes().as_ptr() as _,
        )
    }
}

pub fn promise_batch_action_delete_account(
    promise_index: PromiseIndex,
    beneficiary_id: &[u8],
) {
    unsafe {
        sys::promise_batch_action_delete_account(
            promise_index,
            beneficiary_id.len() as _,
            beneficiary_id.as_ptr() as _,
        )
    }
}

/// If the current function is invoked by a callback we can access the execution results of the
/// promises that caused the callback. This function returns the number of complete and
/// incomplete callbacks.
pub fn promise_results_count() -> u64 {
    unsafe { sys::promise_results_count() }
}
/// If the current function is invoked by a callback we can access the execution results of the
/// promises that caused the callback.
pub fn promise_result(result_idx: u64) -> PromiseResult {
    match unsafe { sys::promise_result(result_idx, ATOMIC_OP_REGISTER) } {
        0 => PromiseResult::NotReady,
        1 => {
            let data = read_register(ATOMIC_OP_REGISTER)
                .expect("Promise result should've returned into register.");
            PromiseResult::Successful(data)
        }
        2 => PromiseResult::Failed,
        _ => unreachable!(),
    }
}
/// Consider the execution result of promise under `promise_idx` as execution result of this
/// function.
pub fn promise_return(promise_idx: PromiseIndex) {
    unsafe { sys::promise_return(promise_idx) }
}

//* HELPERS
/// Reads the value stored under the given key.
pub(crate) fn alloc_storage_read(key: &[u8]) -> Option<Vec<u8>> {
    match unsafe { sys::storage_read(key.len() as _, key.as_ptr() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => Some(read_register(ATOMIC_OP_REGISTER).unwrap_or_else(|| unsafe { sys::panic() })),
        _ => unreachable!(),
    }
}

/// Reads the content of the `register_id`. If register is not used returns `None`.
fn read_register(register_id: u64) -> Option<Vec<u8>> {
    let len = register_len(register_id)?;
    let res = vec![0u8; len as usize];
    unsafe { sys::read_register(register_id, res.as_ptr() as _) };
    Some(res)
}

/// Returns the size of the register. If register is not used returns `None`.
fn register_len(register_id: u64) -> Option<u64> {
    let len = unsafe { sys::register_len(register_id) };
    if len == core::u64::MAX {
        None
    } else {
        Some(len)
    }
}
