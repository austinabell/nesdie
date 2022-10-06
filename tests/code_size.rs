/// Compiles contract to wasm with release configuration and returns the code size.
fn check_example_size(example: &str) -> usize {
    let status = std::process::Command::new("cargo")
        .env("RUSTFLAGS", "-C link-arg=-s")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(format!("./examples/{}/Cargo.toml", example))
        .status()
        .unwrap();
    if !status.success() {
        panic!("building wasm example returned non-zero code {}", status);
    }

    let wasm = std::fs::read(format!(
        "./examples/{}/target/wasm32-unknown-unknown/release/{}.wasm",
        example,
        example.replace('-', "_")
    ))
    .unwrap();

    wasm.len()
}

#[test]
#[ignore = "proxy can't be compiled on stable (alloc error handler)"]
fn proxy_code_size_check() {
    let size = check_example_size("proxy");

    // 3535
    assert!(size < 3600);
}

#[test]
#[cfg_attr(miri, ignore)]
fn raw_contract_code_size_check() {
    let size = check_example_size("raw-contract");

    // 169
    assert!(size < 250);
}

#[test]
#[cfg_attr(miri, ignore)]
fn fungible_token_code_size_check() {
    let size = check_example_size("smol_ft");

    // 1416
    assert!(size < 1500);
}
