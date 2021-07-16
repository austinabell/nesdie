#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_handler))]

pub mod env;
pub mod sys;

mod types;
pub use self::types::{Balance, Gas};

// Set up global allocator by default if wee_alloc feature is enabled and in wasm32 architecture.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    unsafe { core::arch::wasm32::unreachable() }
}

// Update panic handler in wasm32 environments
#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if cfg!(feature = "panic_message") {
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            env::panic_str(s);
        } else {
            env::panic_str("unexpected panic occurred");
        }
    } else {
        unsafe { core::arch::wasm32::unreachable() }
    }
}
