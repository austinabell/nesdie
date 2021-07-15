# nesdie

This is just meant as an experimental `no_std` SDK which will follow similar patterns of `near-sdk-rs` but optimize for minimal code size and operations. This will be a worse devX than the near sdk, but can be used as an alternative to writing bare metal contracts without an SDK.

### Goals for `nesdie`:

- Strict `no_std` for `wasm` binaries
- Little to no code bloat
  - No use of `core::fmt`
  - No `serde` and gate serialization protocols by feature to allow disabling
- Minimize gas costs through less instructions
- Similar amount of boilerplate/structure as `near-sdk-rs` 
- Better error handling in codegen to avoid having to panic or `unwrap` errors
- Don't include local paths in built binary (from panics and asserts)
