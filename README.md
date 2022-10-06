# nesdie

[<img alt="github" src="https://img.shields.io/badge/github-austinabell/nesdie-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/austinabell/nesdie)
[<img alt="crates.io" src="https://img.shields.io/crates/v/nesdie.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/nesdie)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-nesdie-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/nesdie)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/austinabell/nesdie/CI/main?style=for-the-badge" height="20">](https://github.com/austinabell/nesdie/actions?query=branch%3Amain)

This is just meant as an experimental `no_std` SDK which will follow similar patterns of `near-sdk-rs` but optimize for minimal code size and operations. This will be a worse devX than the near sdk, but can be used as an alternative to writing bare metal contracts without an SDK.

## Features

- `wee_alloc` (default): Configures the global allocator by default with [`wee_alloc`](https://github.com/rustwasm/wee_alloc)
- `panic-message`: Configures `panic_handler` to include error details, which will show up on chain. Disabled by default to optimize code size
- `oom-handler`: Configures `alloc_error_handler` to minimize error handling in this case. This feature does not currently work with a `stable` toolchain

### Goals for `nesdie`:

- Strict `no_std` for `wasm` binaries
- Little to no code bloat
  - No use of `core::fmt`
  - No `serde` and gate serialization protocols by feature to allow disabling
- Minimize gas costs through less instructions
- Similar amount of boilerplate/structure as `near-sdk-rs` 
- Better error handling in codegen to avoid having to panic or `unwrap` errors
- Don't include local paths in built binary (from panics and asserts)
