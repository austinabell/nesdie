#![cfg_attr(target_arch = "wasm32", no_std)]

//* Credit to evgenykuzyakov for writing this contract initially
//* https://github.com/near/core-contracts/pull/88

#[cfg(target_arch = "wasm32")]
use nesdie::sys;

const SUPPLY_KEY: &[u8] = b"S";
const LEN: u64 = 32;
const LEN_U64: u64 = 4;
const LEN_U64_USIZE: usize = LEN_U64 as _;

type U256 = [u64];

unsafe fn add(a: &U256, b: &U256, res: &mut U256) {
    let mut old_overflow = false;
    for i in 0..LEN_U64_USIZE {
        let (v, overflow) = a[i].overflowing_add(b[i]);
        let (v, overflow2) = v.overflowing_add(old_overflow as u64);
        old_overflow = overflow || overflow2;
        res[i] = v
    }
    if old_overflow {
        // Overflow
        sys::panic();
    }
}

unsafe fn sub(a: &U256, b: &U256, res: &mut U256) {
    let mut old_underflow = false;
    for i in 0..LEN_U64_USIZE {
        let (v, underflow) = a[i].overflowing_sub(b[i]);
        let (v, underflow2) = v.overflowing_sub(old_underflow as u64);
        old_underflow = underflow2 || underflow;
        res[i] = v
    }
    if old_underflow {
        // Underflow
        sys::panic();
    }
}

/// Initializes the token contract with the total supply given to the owner.
/// Arguments (64 bytes):
/// - 0..32 - `sha256` of the owner address.
/// - 32..64 - U256 of the total supply
#[no_mangle]
pub unsafe fn init() {
    if sys::storage_has_key(SUPPLY_KEY.len() as _, SUPPLY_KEY.as_ptr() as _) == 1 {
        sys::panic();
    }
    let buf = read_input();
    // SUPPLY_KEY
    sys::storage_write(
        SUPPLY_KEY.len() as _,
        SUPPLY_KEY.as_ptr() as _,
        LEN,
        buf.as_ptr() as u64 + LEN,
        0,
    );
    // OWNER BALANCE
    sys::storage_write(LEN, buf.as_ptr() as _, LEN, buf.as_ptr() as u64 + LEN, 0);
}

unsafe fn read_input() -> [u64; LEN_U64_USIZE * 2] {
    sys::input(0);
    let input_len = sys::register_len(0);
    if input_len != LEN * 2 {
        sys::panic();
    }
    let buf = [0u64; LEN_U64_USIZE * 2];
    sys::read_register(0, buf.as_ptr() as _);
    buf
}

/// Transfer the amount from the `sha256(predecessor_account_id)` to the new receiver address.
/// Arguments (64 bytes):
/// - 0..32 - `sha256` of the receiver address.
/// - 32..64 - U256 is transfer amount
#[no_mangle]
pub unsafe fn transfer() {
    let buf = read_input();
    // Read hash of owner's account ID to register 0
    sys::predecessor_account_id(0);
    sys::sha256(u64::MAX, 0, 0);

    // Owner's balance to register 1
    if sys::storage_read(u64::MAX, 0, 1) == 0 {
        // No balance
        sys::panic();
    }

    let owner_balance = [0u64; LEN_U64_USIZE];
    sys::read_register(1, owner_balance.as_ptr() as _);

    let transfer_balance = &buf[LEN_U64_USIZE..LEN_U64_USIZE * 2];
    let mut new_balance = [0u64; LEN_U64_USIZE];
    sub(&owner_balance, transfer_balance, &mut new_balance);

    // Write new owner balance
    sys::storage_write(u64::MAX, 0, LEN, new_balance.as_ptr() as _, 1);

    let receiver_balance = [0u64; LEN_U64_USIZE];
    // Reading and filling receiver_balance.
    if sys::storage_read(LEN, buf.as_ptr() as u64, 1) == 1 {
        sys::read_register(1, receiver_balance.as_ptr() as _);
    }

    // Reusing `new_balance`, since it overwrites all bytes.
    add(&receiver_balance, transfer_balance, &mut new_balance);

    // Writing new owner balance
    sys::storage_write(LEN, buf.as_ptr() as u64, LEN, new_balance.as_ptr() as _, 1);
}

/// Returns the balance of the given address.
/// Arguments (64 bytes):
/// - 0..32 - `sha256` of the address to check the balance.
#[no_mangle]
pub unsafe fn get_balance() {
    sys::input(0);
    let input_len = sys::register_len(0);
    if input_len != LEN {
        sys::panic();
    }

    // Reading receiver_balance and returning it, or returning 0.
    if sys::storage_read(u64::MAX, 0, 1) == 1 {
        sys::value_return(u64::MAX, 1);
    } else {
        let receiver_balance = [0u64; LEN_U64_USIZE];
        sys::value_return(LEN, receiver_balance.as_ptr() as u64);
    }
}
