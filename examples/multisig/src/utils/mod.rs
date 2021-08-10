pub(crate) mod helper_env;
pub(crate) mod json;
mod promise;
mod public_key;

extern crate alloc;

pub(crate) mod types {
    pub use super::promise::*;
    pub use super::public_key::*;
    pub type AccountId = super::alloc::string::String;
    pub type Gas = u64;
    pub type Balance = u128;
    pub type PromiseIndex = u64;

    #[derive(Debug, PartialEq)]
    pub enum PromiseResult {
        /// Current version of the protocol never returns `PromiseResult::NotReady`.
        NotReady,
        Successful(super::alloc::vec::Vec<u8>),
        Failed,
    }
}
