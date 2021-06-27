/// Token denomination type.
pub type Balance = u128;

pub type AccountId = Box<[u8]>;

pub type PublicKey = Box<[u8]>;

pub type Gas = u64;

/// When there is a callback attached to one or more contract calls the execution results of these
/// calls are available to the contract invoked through the callback.
#[derive(Debug, PartialEq)]
pub enum PromiseResult {
    /// Current version of the protocol never returns `PromiseResult::NotReady`.
    NotReady,
    Successful(Vec<u8>),
    Failed,
}
