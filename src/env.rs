use crate::types::Vec;
use crate::{sys, AccountId, Balance, Gas};
use core::mem::size_of;

/// Register used internally for atomic operations. This register is safe to use by the user,
/// since it only needs to be untouched while methods of `Environment` execute, which is guaranteed
/// guest code is not parallel.
const ATOMIC_OP_REGISTER: u64 = 0;
/// Register used to record evicted values from the storage.
const EVICTED_REGISTER: u64 = core::u64::MAX - 1;

/// Key used to store the state of the contract.
const STATE_KEY: &[u8] = b"STATE";

// fn expect_register<T>(option: Option<T>) -> T {
//     option.unwrap_or_else(|| sys_panic())
// }

/// A simple macro helper to read blob value coming from host's method.
macro_rules! try_method_into_register {
    ( $method:ident, $v:expr ) => {{
        unsafe { sys::$method(ATOMIC_OP_REGISTER) };
        read_register(ATOMIC_OP_REGISTER, $v).unwrap_or_else(|_| sys_panic())
    }};
}

/// Same as `try_method_into_register` but expects the data.
macro_rules! method_into_register {
    ( $method:ident, $v:expr ) => {{
        try_method_into_register!($method, $v)
    }};
}

/// Index for a batch promise from within the runtime. Used to combine promises within a contract.
pub struct PromiseIndex(pub u64);

fn sys_panic() -> ! {
    unsafe { sys::panic() }
}

/// Reads the content of the `register_id`. If register is not used or the buffer is not large
/// enough, an error will be returned.
#[allow(clippy::result_unit_err)]
pub fn read_register(register_id: u64, buf: &mut [u8]) -> Result<usize, ()> {
    let len = register_len(register_id).ok_or(())? as usize;
    if buf.len() < len {
        return Err(());
    }
    unsafe { sys::read_register(register_id, buf.as_ptr() as _) };
    Ok(len)
}

/// Returns the size of the register. If register is not used returns `None`.
pub fn register_len(register_id: u64) -> Option<u64> {
    let len = unsafe { sys::register_len(register_id) };
    if len == core::u64::MAX {
        None
    } else {
        Some(len)
    }
}

// ###############
// # Context API #
// ###############

// TODO eval this API before releasing
/// The id of the account that owns the current contract.
pub fn current_account_id() -> AccountId {
    let mut a = Vec::<u8, 64>::new();
    let len = method_into_register!(current_account_id, a.as_mut());
    // Update length for size written
    unsafe {
        a.set_len(len);
        // Fine to cast as account id, should be validated by runtime
        AccountId::new_raw(a)
    }
}

/// Current block index.
pub fn block_index() -> u64 {
    unsafe { sys::block_index() }
}

/// Current block timestamp, i.e, number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC.
pub fn block_timestamp() -> u64 {
    unsafe { sys::block_timestamp() }
}

/// Current epoch height.
pub fn epoch_height() -> u64 {
    unsafe { sys::epoch_height() }
}

/// Current total storage usage of this smart contract that this account would be paying for.
pub fn storage_usage() -> u64 {
    unsafe { sys::storage_usage() }
}

// #################
// # Economics API #
// #################
/// The balance attached to the given account. This includes the attached_deposit that was
/// attached to the transaction
pub fn account_balance() -> Balance {
    let data = [0u8; size_of::<Balance>()];
    unsafe { sys::account_balance(data.as_ptr() as u64) };
    Balance::from_le_bytes(data)
}

/// The balance locked for potential validator staking.
pub fn account_locked_balance() -> Balance {
    let data = [0u8; size_of::<Balance>()];
    unsafe { sys::account_locked_balance(data.as_ptr() as u64) };
    Balance::from_le_bytes(data)
}

/// The balance that was attached to the call that will be immediately deposited before the
/// contract execution starts
pub fn attached_deposit() -> Balance {
    let data = [0u8; size_of::<Balance>()];
    unsafe { sys::attached_deposit(data.as_ptr() as u64) };
    Balance::from_le_bytes(data)
}

/// The amount of gas attached to the call that can be used to pay for the gas fees.
pub fn prepaid_gas() -> Gas {
    unsafe { sys::prepaid_gas() }
}

/// The gas that was already burnt during the contract execution (cannot exceed `prepaid_gas`)
pub fn used_gas() -> Gas {
    unsafe { sys::used_gas() }
}

// ############
// # Math API #
// ############
// /// Get random seed from the register.
// pub fn random_seed() -> Vec<u8> {
//     method_into_register!(random_seed)
// }

/// Hashes the random sequence of bytes using sha256.
pub fn sha256(value: &[u8]) -> [u8; 32] {
    unsafe { sys::sha256(value.len() as _, value.as_ptr() as _, ATOMIC_OP_REGISTER) };
    let mut hash = [0u8; 32];

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| sys_panic());
    hash
}

/// Hashes the random sequence of bytes using keccak256.
pub fn keccak256(value: &[u8]) -> [u8; 32] {
    unsafe { sys::keccak256(value.len() as _, value.as_ptr() as _, ATOMIC_OP_REGISTER) };
    let mut hash = [0u8; 32];

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| sys_panic());
    hash
}

/// Hashes the random sequence of bytes using keccak512.
pub fn keccak512(value: &[u8]) -> [u8; 64] {
    unsafe { sys::keccak512(value.len() as _, value.as_ptr() as _, ATOMIC_OP_REGISTER) };
    let mut hash = [0u8; 64];

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| sys_panic());
    hash
}

// ###############
// # Validator API #
// ###############

/// For a given account return its current stake. If the account is not a validator, returns 0.
pub fn validator_stake(account_id: &str) -> Balance {
    let data = [0u8; size_of::<Balance>()];
    unsafe {
        sys::validator_stake(
            account_id.len() as _,
            account_id.as_ptr() as _,
            data.as_ptr() as u64,
        )
    };
    Balance::from_le_bytes(data)
}

/// Returns the total stake of validators in the current epoch.
pub fn validator_total_stake() -> Balance {
    let data = [0u8; size_of::<Balance>()];
    unsafe { sys::validator_total_stake(data.as_ptr() as u64) };
    Balance::from_le_bytes(data)
}

// #####################
// # Miscellaneous API #
// #####################
/// Sets the blob of data as the return value of the contract.
pub fn value_return(value: &[u8]) {
    unsafe { sys::value_return(value.len() as _, value.as_ptr() as _) }
}

/// Terminates the execution of the program with the UTF-8 encoded message.
pub fn panic_str(message: &str) -> ! {
    unsafe { sys::panic_utf8(message.len() as _, message.as_ptr() as _) }
}
/// Log the UTF-8 encodable message.
pub fn log_str(message: &str) {
    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    eprintln!("{}", message);
    unsafe { sys::log_utf8(message.len() as _, message.as_ptr() as _) }
}

// ###############
// # Storage API #
// ###############
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
        _ => sys_panic(),
    }
}
/// Reads the value stored under the given key.
pub fn storage_read(key: &[u8], buf: &mut [u8]) -> Option<usize> {
    match unsafe { sys::storage_read(key.len() as _, key.as_ptr() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => Some(read_register(ATOMIC_OP_REGISTER, buf).unwrap_or_else(|_| sys_panic())),
        _ => sys_panic(),
    }
}
/// Removes the value stored under the given key.
/// If key-value existed returns `true`, otherwise `false`.
pub fn storage_remove(key: &[u8]) -> bool {
    match unsafe { sys::storage_remove(key.len() as _, key.as_ptr() as _, EVICTED_REGISTER) } {
        0 => false,
        1 => true,
        _ => sys_panic(),
    }
}
/// Reads the most recent value that was evicted with `storage_write` or `storage_remove` command.
pub fn storage_get_evicted(buf: &mut [u8]) -> Option<usize> {
    read_register(EVICTED_REGISTER, buf).ok()
}
/// Checks if there is a key-value in the storage.
pub fn storage_has_key(key: &[u8]) -> bool {
    match unsafe { sys::storage_has_key(key.len() as _, key.as_ptr() as _) } {
        0 => false,
        1 => true,
        _ => sys_panic(),
    }
}

// ############################################
// # Saving and loading of the contract state #
// ############################################
/// Load the state of the given object.
/// Read raw bytes under the static state key.
pub fn state_read_raw(buf: &mut [u8]) -> Option<usize> {
    storage_read(STATE_KEY, buf)
}

/// Write bytes under the static state key.
pub fn state_write_raw(data: &[u8]) {
    storage_write(STATE_KEY, data);
}

/// Returns `true` if the contract state exists and `false` otherwise.
pub fn state_exists() -> bool {
    storage_has_key(STATE_KEY)
}

//* Promises

// Creates a promise that will execute a method on account with given arguments and attaches
/// the given amount and gas.
pub fn promise_create(
    account_id: &str,
    method_name: &str,
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) -> PromiseIndex {
    unsafe {
        PromiseIndex(sys::promise_create(
            account_id.len() as _,
            account_id.as_ptr() as _,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        ))
    }
}

/// Attaches the callback that is executed after promise pointed by `promise_idx` is complete.
pub fn promise_then(
    promise_idx: PromiseIndex,
    account_id: &str,
    method_name: &str,
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) -> PromiseIndex {
    unsafe {
        PromiseIndex(sys::promise_then(
            promise_idx.0,
            account_id.len() as _,
            account_id.as_ptr() as _,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        ))
    }
}

// TODO consider API, currently requires alloc
// /// Creates a new promise which completes when time all promises passed as arguments complete.
// pub fn promise_and(promise_indices: &[PromiseIndex]) -> PromiseIndex {
//     let mut data = vec![0u8; promise_indices.len() * size_of::<PromiseIndex>()];
//     for i in 0..promise_indices.len() {
//         data[i * size_of::<PromiseIndex>()..(i + 1) * size_of::<PromiseIndex>()]
//             .copy_from_slice(&promise_indices[i].to_le_bytes());
//     }
//     unsafe {
//         PromiseIndex(sys::promise_and(
//             data.as_ptr() as _,
//             promise_indices.len() as _,
//         ))
//     }
// }

/// Create a batch promise and return the index of that promise.
pub fn promise_batch_create(account_id: &str) -> PromiseIndex {
    unsafe {
        PromiseIndex(sys::promise_batch_create(
            account_id.len() as _,
            account_id.as_ptr() as _,
        ))
    }
}

/// Schedule a promise after the provided promise index.
pub fn promise_batch_then(promise_index: PromiseIndex, account_id: &str) -> PromiseIndex {
    unsafe {
        PromiseIndex(sys::promise_batch_then(
            promise_index.0,
            account_id.len() as _,
            account_id.as_ptr() as _,
        ))
    }
}

/// Create account with the batch promise.
pub fn promise_batch_action_create_account(promise_index: PromiseIndex) {
    unsafe { sys::promise_batch_action_create_account(promise_index.0) }
}

/// Deploy contract with the batch promise.
pub fn promise_batch_action_deploy_contract(promise_index: u64, code: &[u8]) {
    unsafe {
        sys::promise_batch_action_deploy_contract(
            promise_index,
            code.len() as _,
            code.as_ptr() as _,
        )
    }
}

/// Call a function within the batch promise.
pub fn promise_batch_action_function_call(
    promise_index: PromiseIndex,
    method_name: &str,
    arguments: &[u8],
    amount: Balance,
    gas: Gas,
) {
    unsafe {
        sys::promise_batch_action_function_call(
            promise_index.0,
            method_name.len() as _,
            method_name.as_ptr() as _,
            arguments.len() as _,
            arguments.as_ptr() as _,
            &amount as *const Balance as _,
            gas,
        )
    }
}

/// Transfer tokens with the promise.
pub fn promise_batch_action_transfer(promise_index: PromiseIndex, amount: Balance) {
    unsafe { sys::promise_batch_action_transfer(promise_index.0, &amount as *const Balance as _) }
}

/// Stake tokens with the promise.
pub fn promise_batch_action_stake(promise_index: PromiseIndex, amount: Balance, public_key: &[u8]) {
    unsafe {
        sys::promise_batch_action_stake(
            promise_index.0,
            &amount as *const Balance as _,
            public_key.len() as _,
            public_key.as_ptr() as _,
        )
    }
}

/// Add full access key with batch promise.
pub fn promise_batch_action_add_key_with_full_access(
    promise_index: PromiseIndex,
    public_key: &[u8],
    nonce: u64,
) {
    unsafe {
        sys::promise_batch_action_add_key_with_full_access(
            promise_index.0,
            public_key.len() as _,
            public_key.as_ptr() as _,
            nonce,
        )
    }
}

/// Add access key with only function call priviledges
pub fn promise_batch_action_add_key_with_function_call(
    promise_index: PromiseIndex,
    public_key: &[u8],
    nonce: u64,
    allowance: Balance,
    receiver_id: &str,
    method_names: &str,
) {
    unsafe {
        sys::promise_batch_action_add_key_with_function_call(
            promise_index.0,
            public_key.len() as _,
            public_key.as_ptr() as _,
            nonce,
            &allowance as *const Balance as _,
            receiver_id.len() as _,
            receiver_id.as_ptr() as _,
            method_names.len() as _,
            method_names.as_ptr() as _,
        )
    }
}

/// Delete access key with batch promise.
pub fn promise_batch_action_delete_key(promise_index: PromiseIndex, public_key: &[u8]) {
    unsafe {
        sys::promise_batch_action_delete_key(
            promise_index.0,
            public_key.len() as _,
            public_key.as_ptr() as _,
        )
    }
}

/// Delete account with batch promise.
pub fn promise_batch_action_delete_account(promise_index: PromiseIndex, beneficiary_id: &str) {
    unsafe {
        sys::promise_batch_action_delete_account(
            promise_index.0,
            beneficiary_id.len() as _,
            beneficiary_id.as_ptr() as _,
        )
    }
}

// #####################################
// # Parameters exposed by the runtime #
// #####################################

/// Price per 1 byte of storage from mainnet genesis config.
/// TODO: will be using the host function when it will be available.
const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;

/// Returns the storage cost per byte.
pub fn storage_byte_cost() -> Balance {
    STORAGE_PRICE_PER_BYTE
}
