use crate::{sys, Balance, Gas};
use core::mem::size_of;

/// Register used internally for atomic operations. This register is safe to use by the user,
/// since it only needs to be untouched while methods of `Environment` execute, which is guaranteed
/// guest code is not parallel.
const ATOMIC_OP_REGISTER: u64 = 0;
/// Register used to record evicted values from the storage.
const EVICTED_REGISTER: u64 = core::u64::MAX - 1;

/// Key used to store the state of the contract.
const STATE_KEY: &[u8] = b"STATE";

// Update panic handler in wasm32 environments
#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if cfg!(feature = "panic_message") {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            panic_str(s);
        } else {
            panic_str("unexpected panic occurred");
        }
    } else {
        unsafe { core::arch::wasm32::unreachable() }
    }
}

/// Reads the content of the `register_id`. If register is not used or the buffer is not large
/// enough, an error will be returned.
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

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| unreachable!());
    hash
}

/// Hashes the random sequence of bytes using keccak256.
pub fn keccak256(value: &[u8]) -> [u8; 32] {
    unsafe { sys::keccak256(value.len() as _, value.as_ptr() as _, ATOMIC_OP_REGISTER) };
    let mut hash = [0u8; 32];

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| unreachable!());
    hash
}

/// Hashes the random sequence of bytes using keccak512.
pub fn keccak512(value: &[u8]) -> [u8; 64] {
    unsafe { sys::keccak512(value.len() as _, value.as_ptr() as _, ATOMIC_OP_REGISTER) };
    let mut hash = [0u8; 64];

    read_register(ATOMIC_OP_REGISTER, &mut hash).unwrap_or_else(|_| unreachable!());
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
    unreachable!()
}
/// Log the UTF-8 encodable message.
pub fn log_str(message: &str) {
    #[cfg(all(
        debug_assertions,
        target_arch = "wasm32-unknown-unknown",
        not(target_arch = "wasm32")
    ))]
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
        _ => unreachable!(),
    }
}
/// Reads the value stored under the given key.
pub fn storage_read(key: &[u8], buf: &mut [u8]) -> Option<usize> {
    match unsafe { sys::storage_read(key.len() as _, key.as_ptr() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => Some(read_register(ATOMIC_OP_REGISTER, buf).unwrap_or_else(|_| unreachable!())),
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
pub fn storage_get_evicted(buf: &mut [u8]) -> Option<usize> {
    read_register(EVICTED_REGISTER, buf).ok()
}
/// Checks if there is a key-value in the storage.
pub fn storage_has_key(key: &[u8]) -> bool {
    match unsafe { sys::storage_has_key(key.len() as _, key.as_ptr() as _) } {
        0 => false,
        1 => true,
        _ => unreachable!(),
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
    storage_write(STATE_KEY, &data);
}

/// Returns `true` if the contract state exists and `false` otherwise.
pub fn state_exists() -> bool {
    storage_has_key(STATE_KEY)
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
