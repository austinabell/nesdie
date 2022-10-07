// NOTE: heapless currently needs to be vendored since std is being pulled
// in through `heapless/hash32/byteorder`. Also, they don't have a way to
// unsafely construct a String from Vec without requiring a copy (optimization).
mod heapless;
pub(crate) use self::heapless::Vec;

/// Token denomination type.
pub type Balance = u128;

/// Gas units which are used for computation and storage within the runtime.
pub type Gas = u64;

// TODO maybe don't want to export this -- check before releasing
/// Account identifier
pub type AccountId = heapless::String<64>;
