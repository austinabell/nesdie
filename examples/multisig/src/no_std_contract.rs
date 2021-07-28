use super::*;
use serde_json as json;

const ERR_DESERIALIZE_INPUT: &str = "failed to deserialize input JSON";
const ERR_UTF8: &str = "utf8 error";

#[no_mangle]
pub extern "C" fn new() {
    // helper_env::setup_panic_hook();
    if env::attached_deposit() != 0 {
        env::panic_str("Method doesn't accept deposit");
    }

    #[derive(Serialize, Deserialize)]
    struct Input {
        num_confirmations: u32,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { num_confirmations } = json::from_str(
        core::str::from_utf8(&input).unwrap_or_else(|_| env::panic_str(ERR_UTF8)),
    )
    .unwrap_or_else(|_| env::panic_str(ERR_DESERIALIZE_INPUT));

    let contract = MultiSigContract::new(num_confirmations);
    helper_env::state_write(&contract);
}
