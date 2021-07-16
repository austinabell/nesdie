# nesdie

This is just meant as an experimental `no_std` SDK which will follow similar patterns of `near-sdk-rs` but optimize for minimal code size and operations. This will be a worse devX than the near sdk, but can be used as an alternative to writing bare metal contracts without an SDK.

## Features

- `wee_alloc` (default): Configures the global allocator by default with [`wee_alloc`](https://github.com/rustwasm/wee_alloc)
- `panic-message`: Configures `panic_handler` to include error details, which will show up on chain. Disabled by default to optimize code size

### Goals for `nesdie`:

- Strict `no_std` for `wasm` binaries
- Little to no code bloat
  - No use of `core::fmt`
  - No `serde` and gate serialization protocols by feature to allow disabling
- Minimize gas costs through less instructions
- Similar amount of boilerplate/structure as `near-sdk-rs` 
- Better error handling in codegen to avoid having to panic or `unwrap` errors
- Don't include local paths in built binary (from panics and asserts)
