#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod env;
pub mod sys;

mod types;
pub use self::types::{AccountId, Balance, Gas, PromiseResult, PublicKey};

// Set up global allocator by default if wee_alloc feature is enabled and in wasm32 architecture.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
