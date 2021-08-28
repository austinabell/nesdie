#![cfg_attr(target_arch = "wasm32", no_std)]

const MIGRATE_FUNCTION_NAME: &str = "migrate";
/// Gas for calling migration call.
const GAS_FOR_MIGRATE_CALL: u64 = 5_000_000_000_000;

use nesdie::{env, sys};

#[no_mangle]
pub fn migrate() {
    env::log_str("performing arbitrary migration logic");
}

#[no_mangle]
pub fn some_new_function() {
    env::log_str("can call some new function now!");
}

#[no_mangle]
pub fn upgrade() {
    //* Might want to assert a contract owner, otherwise anyone can upgrade

    let current_account_id = env::current_account_id();

    // Put input bytes into register 1. This should be the wasm contract
    unsafe { sys::input(1) };

    let promise_id = env::promise_batch_create(current_account_id.as_str());
    unsafe { sys::promise_batch_action_deploy_contract(promise_id.0, u64::MAX as _, 1) };

    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
    unsafe {
        sys::promise_batch_action_function_call(
            promise_id.0,
            MIGRATE_FUNCTION_NAME.len() as _,
            MIGRATE_FUNCTION_NAME.as_ptr() as _,
            0 as _,
            0 as _,
            0 as _,
            attached_gas,
        )
    };
}
