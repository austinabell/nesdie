//! `nesdie` is an experimental `no_std` SDK for NEAR Protocol which optimizes for small compiled
//! WASM code size.
//!
//! This SDK is setup like a domain-specific language and configures all low level boilerplate
//! of the contract by default.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(
    all(target_arch = "wasm32", feature = "oom-handler"),
    feature(alloc_error_handler)
)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(dead_code, unused_mut)]
#![warn(missing_docs)]

/// Higher level environment functions which act as a safe wrapper around [`sys`].
pub mod env;
/// Host functions available to a NEAR contract through the runtime the contract is running inside.
pub mod sys;

#[cfg(not(target_arch = "wasm32"))]
/// Mock utilities used for testing and overriding the syscall interface for contracts.
pub mod mock;

mod types;
pub use self::types::{AccountId, Balance, Gas};

// Set up global allocator by default if wee_alloc feature is enabled and in wasm32 architecture.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(all(not(feature = "std"), feature = "oom-handler", target_arch = "wasm32"))]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    unsafe { core::arch::wasm32::unreachable() }
}

// Update panic handler in wasm32 environments
#[cfg(all(target_arch = "wasm32", not(feature = "std")))]
#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if cfg!(feature = "panic-message") {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            env::panic_str(s);
        } else {
            env::panic_str("unexpected panic occurred");
        }
    } else {
        unsafe { core::arch::wasm32::unreachable() }
    }
}
