#![cfg_attr(target_arch = "wasm32", no_std)]

#[cfg(target_arch = "wasm32")]
use nesdie::env;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn test_function() {
    env::log_str("test call");
}
