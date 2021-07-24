mod cache_entry;
mod once_cell;
mod stable_map;

pub(crate) use self::cache_entry::{CacheEntry, EntryState};
pub(crate) use self::once_cell::OnceCell;
pub(crate) use self::stable_map::StableMap;

use nesdie::sys;

const ATOMIC_OP_REGISTER: u64 = 0;

pub(crate) fn append_slice(a: &[u8], b: &[u8]) -> Vec<u8> {
    [a, b].concat()
}

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
    if len == std::u64::MAX {
        None
    } else {
        Some(len)
    }
}
