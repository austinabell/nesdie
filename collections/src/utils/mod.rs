use crate::lib::Vec;
use core::convert::TryInto;
use nesdie::{env, sys};

const ATOMIC_OP_REGISTER: u64 = core::u64::MAX - 1;
const EVICTED_REGISTER: u64 = core::u64::MAX - 2;

/// Reads the value stored under the given key.
pub(crate) fn alloc_storage_read(key: &[u8]) -> Option<Vec<u8>> {
    match unsafe { sys::storage_read(key.len() as _, key.as_ptr() as _, ATOMIC_OP_REGISTER) } {
        0 => None,
        1 => {
            Some(read_register_alloc(ATOMIC_OP_REGISTER).unwrap_or_else(|| unsafe { sys::panic() }))
        }
        _ => unreachable!(),
    }
}

/// Reads the content of the `register_id`. If register is not used returns `None`.
pub fn read_register_alloc(register_id: u64) -> Option<Vec<u8>> {
    // Get register length and convert to a usize. The max register size in config is much less
    // than the u32 max so the abort should never be hit, but is there for safety because there
    // would be undefined behaviour during `read_register` if the buffer length is truncated.
    let len: usize = register_len(register_id)?
        .try_into()
        .unwrap_or_else(|_| env::abort());

    // Initialize buffer with capacity.
    let mut buffer = Vec::with_capacity(len);

    // Read register into buffer.
    //* SAFETY: This is safe because the buffer is initialized with the exact capacity of the
    //*         register that is being read from.
    unsafe {
        sys::read_register(register_id, buffer.as_mut_ptr() as u64);

        // Set updated length after writing to buffer.
        buffer.set_len(len);
    }
    Some(buffer)
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

/// Removes the value stored under the given key.
/// If key-value existed returns `true`, otherwise `false`.
pub(crate) fn storage_remove_alloc(key: &[u8]) -> Option<Vec<u8>> {
    let removed = env::storage_remove(&key);
    if removed {
        read_register_alloc(EVICTED_REGISTER)
    } else {
        None
    }
}
