mod context;
mod external;
mod mocked_blockchain;
mod receipt;

pub(crate) use self::external::SdkExternal;
pub use self::mocked_blockchain::MockedBlockchain;
pub use self::receipt::{Receipt, VmAction};
pub use context::VMContextBuilder;
use core::cell::RefCell;

// TODO I'd like to remove this export
pub use near_vm_logic::VMContext;

thread_local! {
    /// Low-level blockchain interface wrapped by the environment. Prefer using `env::*` and
    /// `testing_env` for interacting with the real and fake blockchains.
    static BLOCKCHAIN_INTERFACE: RefCell<MockedBlockchain>
         = RefCell::new(MockedBlockchain::default());
}

/// Initializes a testing environment to mock interactions which would otherwise go through a
/// validator node. This macro will initialize or overwrite the [`MockedBlockchain`]
/// instance for interactions from a smart contract.
///
/// There are five parameters that can be accepted to configure the interface with a
/// [`MockedBlockchain`], in this order:
/// - `context`: [`VMContext`] which contains some core information about
/// the blockchain and message data which can be used from the smart contract.
/// - `config` (optional): [`VMConfig`] which contains some additional information
/// about the VM to configure parameters not directly related to the transaction being executed.
/// - `fee_config`(optional): [`RuntimeFeesConfig`] which configures the
/// fees for execution and storage of transactions.
/// - `validators`(optional): a [`HashMap`]<[`AccountId`], [`Balance`]> mocking the
/// current validators of the blockchain.
/// - `promise_results`(optional): a [`Vec`] of [`PromiseResult`] which mocks the results
/// of callback calls during the execution.
///
/// Any argument not included will use the default implementation of each.
///
/// # Example use
///
/// ```
/// use near_sdk::testing_env;
/// use near_sdk::test_utils::{accounts, VMContextBuilder};
/// use near_sdk::{VMConfig, RuntimeFeesConfig};
/// use std::collections::HashMap;
///
/// # fn main() {
/// // Initializing some context is required
/// let context = VMContextBuilder::new().signer_account_id(accounts(0)).build();
///
/// // Build with just the base context
/// testing_env!(context.clone());
///
/// // Or include arguments up to the five optional
/// testing_env!(
///     context,
///     VMConfig::default(),
///     RuntimeFeesConfig::default(),
///     HashMap::default(),
///     Vec::default(),
/// );
/// # }
/// ```
///
/// [`MockedBlockchain`]: crate::mock::MockedBlockchain
/// [`VMContext`]: crate::VMContext
/// [`VMConfig`]: crate::VMConfig
/// [`RuntimeFeesConfig`]: crate::RuntimeFeesConfig
/// [`AccountId`]: crate::AccountId
/// [`Balance`]: crate::Balance
/// [`PromiseResult`]: crate::PromiseResult
/// [`HashMap`]: std::collections::HashMap
#[macro_export]
macro_rules! testing_env {
    ($context:expr, $config:expr, $fee_config:expr, $validators:expr, $promise_results:expr $(,)?) => {
        $crate::mock::set_mocked_blockchain($crate::mock::MockedBlockchain::new(
            $context,
            $config,
            $fee_config,
            $promise_results,
            $crate::mock::with_mocked_blockchain(|b| ::core::mem::take(b.storage_mut())),
            $validators,
            None,
        ));
    };
    ($context:expr, $config:expr, $fee_config:expr, $validators:expr $(,)?) => {
        testing_env!(
            $context,
            $config,
            $fee_config,
            $validators,
            Default::default()
        );
    };

    ($context:expr, $config:expr, $fee_config:expr $(,)?) => {
        testing_env!($context, $config, $fee_config, Default::default());
    };
    ($context:expr) => {
        testing_env!($context, Default::default(), Default::default());
    };
}

/// Replaces the mocked blockchain instance with the passed in one.
pub fn set_mocked_blockchain(blockchain_interface: MockedBlockchain) {
    crate::mock::with_mocked_blockchain(|b| {
        *b = blockchain_interface;
    })
}

/// Initializes a new mocked blockchain with the given context. This keeps existing storage.
pub fn new_context(context: VMContext) {
    with_mocked_blockchain(|b| {
        let storage = core::mem::take(b.storage_mut());
        *b = MockedBlockchain::new(
            context,
            Default::default(),
            Default::default(),
            vec![],
            storage,
            Default::default(),
            None,
        )
    });
}

/// Perform function on a mutable reference to the [`MockedBlockchain`]. This can only be used
/// inside tests.
pub fn with_mocked_blockchain<F, R>(f: F) -> R
where
    F: FnOnce(&mut MockedBlockchain) -> R,
{
    BLOCKCHAIN_INTERFACE.with(|b| f(&mut b.borrow_mut()))
}
