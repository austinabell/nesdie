#![cfg_attr(target_arch = "wasm32", no_std)]

extern crate alloc;

use alloc::vec;

#[cfg(target_arch = "wasm32")]
use nesdie::sys;

/// Check that predecessor of given account if suffix of given account.
fn assert_predecessor() {
    unsafe {
        sys::current_account_id(0);
        let current_account = vec![0u8; sys::register_len(0) as usize];
        sys::read_register(0, current_account.as_ptr() as *const u64 as u64);
        sys::predecessor_account_id(1);
        let mut predecessor_account = vec![0u8; (sys::register_len(1) + 1) as usize];
        predecessor_account[0] = b'.';
        sys::read_register(1, predecessor_account[1..].as_ptr() as *const u64 as u64);
        if !current_account.ends_with(&predecessor_account) {
            sys::panic();
        }
    }
}

fn slice_to_u64(s: &[u8]) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(s);
    u64::from_le_bytes(word)
}

fn slice_to_u32(s: &[u8]) -> u32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(s);
    u32::from_le_bytes(word)
}

/// This proxies passed call.
/// Checks that predecessor is suffix of the given account.
/// <gas:64><amount:u128><receiver_len:u32><receiver_id:bytes><method_name_len:u32><method_name:bytes><args_len:u32><args:bytes>
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub fn call() {
    assert_predecessor();
    unsafe {
        sys::input(2);
        let data = vec![0u8; sys::register_len(2) as usize];
        sys::read_register(2, data.as_ptr() as *const u64 as u64);
        let gas = slice_to_u64(&data[..8]);
        let amount = &data[8..24]; // as u128;
        let receiver_len = slice_to_u32(&data[24..28]) as usize;
        let method_name_len = slice_to_u32(&data[28 + receiver_len..32 + receiver_len]) as usize;
        let args_len = slice_to_u32(
            &data[32 + receiver_len + method_name_len..36 + receiver_len + method_name_len],
        ) as usize;
        let id = sys::promise_batch_create(receiver_len as _, data.as_ptr() as u64 + 28);
        sys::promise_batch_action_function_call(
            id,
            method_name_len as _,
            data.as_ptr() as u64 + 32 + receiver_len as u64,
            args_len as _,
            data.as_ptr() as u64 + 36 + (receiver_len + method_name_len) as u64,
            amount.as_ptr() as _,
            gas,
        );
    }
}

/// Transfers given amount of $NEAR to given account.
/// Input format <amount:u128><receiver_id:bytes>
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub fn transfer() {
    assert_predecessor();
    unsafe {
        sys::input(2);
        let data = vec![0u8; sys::register_len(2) as usize];
        sys::read_register(2, data.as_ptr() as *const u64 as u64);
        let id = sys::promise_batch_create((data.len() - 16) as _, data.as_ptr() as u64 + 16);
        sys::promise_batch_action_transfer(id, data.as_ptr() as _);
    }
}

/// This allows to update the contract on this account.
/// Checks that predecessor is suffix of the given account.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub fn update() {
    assert_predecessor();
    unsafe {
        let id = sys::promise_batch_create(u64::MAX as _, 0 as _);
        sys::input(2);
        sys::promise_batch_action_deploy_contract(id, u64::MAX as _, 2 as _);
    }
}
