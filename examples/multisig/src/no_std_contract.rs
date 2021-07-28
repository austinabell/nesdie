use super::*;
use serde_json as json;

const ERR_DESERIALIZE_INPUT: &str = "failed to deserialize input";
const ERR_SERIALIZE_RETURN: &str = "failed to serialize return value";
const ERR_UTF8: &str = "utf8 error";
const ERR_NO_DEPOSIT: &str = "Method doesn't accept deposit";

fn deserialize_input<'a, T>(input: &'a [u8]) -> T
where
    T: Deserialize<'a>,
{
    json::from_str(core::str::from_utf8(input).unwrap_or_else(|_| env::panic_str(ERR_UTF8)))
        .unwrap_or_else(|_| env::panic_str(ERR_DESERIALIZE_INPUT))
}

#[no_mangle]
pub extern "C" fn new() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }

    #[derive(Deserialize)]
    struct Input {
        num_confirmations: u32,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { num_confirmations } = deserialize_input(&input);

    let contract = MultiSigContract::new(num_confirmations);
    helper_env::state_write(&contract);
}

#[no_mangle]
pub extern "C" fn add_request() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request: MultiSigRequest,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request }: Input = deserialize_input(&input);
    let mut contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.add_request(request);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
    helper_env::state_write(&contract)
}

#[no_mangle]
pub extern "C" fn add_request_and_confirm() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request: MultiSigRequest,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request }: Input = deserialize_input(&input);
    let mut contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.add_request_and_confirm(request);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
    helper_env::state_write(&contract)
}

#[no_mangle]
pub extern "C" fn delete_request() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request_id: RequestId,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request_id }: Input = deserialize_input(&input);
    let mut contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    contract.delete_request(request_id);
    helper_env::state_write(&contract)
}

#[no_mangle]
pub extern "C" fn confirm() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request_id: RequestId,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request_id }: Input = deserialize_input(&input);
    let mut contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.confirm(request_id);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
    helper_env::state_write(&contract)
}

#[no_mangle]
pub extern "C" fn get_request() {
    // TODO codegen doesn't check deposit on read functions, but probably should?
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request_id: RequestId,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request_id }: Input = deserialize_input(&input);
    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.get_request(request_id);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}

#[no_mangle]
pub extern "C" fn get_num_requests_pk() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        public_key: PublicKey,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { public_key }: Input = deserialize_input(&input);
    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.get_num_requests_pk(public_key);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}

#[no_mangle]
pub extern "C" fn list_request_ids() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }

    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.list_request_ids();
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}

#[no_mangle]
pub extern "C" fn get_confirmations() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }
    #[derive(Deserialize)]
    struct Input {
        request_id: RequestId,
    }

    let input = helper_env::input().unwrap_or_else(|| unreachable!());
    let Input { request_id }: Input = deserialize_input(&input);
    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.get_confirmations(request_id);
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}

#[no_mangle]
pub extern "C" fn get_num_confirmations() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }

    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.get_num_confirmations();
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}

#[no_mangle]
pub extern "C" fn get_request_nonce() {
    if env::attached_deposit() != 0 {
        env::panic_str(ERR_NO_DEPOSIT);
    }

    let contract: MultiSigContract = helper_env::state_read().unwrap_or_default();
    let result = contract.get_request_nonce();
    let result = json::to_vec(&result).unwrap_or_else(|_| env::panic_str(ERR_SERIALIZE_RETURN));
    env::value_return(&result);
}
