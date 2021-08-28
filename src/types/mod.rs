mod heapless;

/// Token denomination type.
pub type Balance = u128;

/// Gas units which are used for computation and storage within the runtime.
pub type Gas = u64;

// TODO maybe don't want to export this -- check before releasing
/// Account identifier
pub type AccountId = heapless::String<64>;
