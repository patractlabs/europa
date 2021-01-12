## Europa `pallet-contracts` sandbox
When regarding Europa as a sandbox to run [ink!](https://github.com/paritytech/ink), [ask!](https://github.com/patractlabs/ask) and [Solang](https://github.com/hyperledger-labs/solang) contracts, 
it is a very awesome and useful tool. We design many features to help developers to locate their bugs and errors in their contracts.

Though the contracts framework (like `ink!`) may already provide a completed mocked environment to run test case, but 
the mocked environment is different from the chain environment eventually. For example the mocked environment is hard to
debug the situation that one contract call another contract, however in fact, those cases are common in defi contracts. 
And in another word, the prosperous ecology is not relayed on a single contract, it relies on the combination of different
contracts.

### Features
* Contract execution event tracker
* WASM panic backtrace
* Self ChainExtension 
    * Contract logger
    * ZKP feature

We do following things to support those features:
* Modification on the `pallet-contracts` layer
  
    By adding trace during the execution of the contract by `pallet-contracts`, 
    the information in the contract layer is recorded, especially when a contract all another contract. The recorded 
    information is mapping with the contract stack, could be analysed the relationship between contracts. On the other 
    hand, the error message of calling WASM execution is improved.
  
* Modification on the `wasmi` layer
    We have provided the backtrace function of recording wasm execution for `wasmi`, and provided support for 
    `parity-wasm`, `pwasm-utils`, and `cargo-contract` during wasm processing of the contract contains the function of 
    the name section. The name section also will provide the basic requirement to debug WASM contracts by gdb/lldb in future. 
  
* ChainExtension:
    * Contract logger
        We integrate the lib which is provided by PatractLabs: [ink-log](https://github.com/patractlabs/ink-log). This 
        lib pass log data from contract to Europa through ChainExtensions.
    * ZKP feature
        We integrate the lib which is provided by PatractLabs: [megaclite](https://github.com/patractlabs/megaclite). 
        This lib providers different curves to support basic function to run Groth16 algorithm in contracts.

### Prepare
For using all features when running contracts in Europa, we advice developers use [PatractLabs's `cargo-contract`](https://github.com/paritytech/cargo-contract)
to compile ink! contract, until [this pr](https://github.com/paritytech/cargo-contract/pull/131) could be merged by parity.

In PatractLabs's `cargo-contract`, we will contain the "name section" while compile

#### 1. Contract execution event tracker
In our forked `pallet-contracts`, we define the struct `NestedRuntime` to track the event when developers execute contracts:
```rust
/// Record the contract execution context.
pub struct NestedRuntime {
	/// Current depth
    depth: usize,
	/// The current contract execute result
	ext_result: ExecResultTrace,
	/// The value in sandbox successful result
	sandbox_result_ok: Option<ReturnValue>,
	/// Who call the current contract
    caller: AccountId32,
	/// The account of the current contract
    self_account: Option<AccountId32>,
	/// The input selector
    selector: Option<HexVec>,
	/// The input arguments
    args: Option<HexVec>,
	/// The value in call or the endowment in instantiate
    value: u128,
	/// The gas limit when this contract is called
    gas_limit: Gas,
	/// The gas left when this contract return
    gas_left: Gas,
	/// The host function call stack
    env_trace: EnvTraceList,
	/// The error in wasm
    wasm_error: Option<WasmErrorWrapper>,
	/// The trap in host function execution
    trap_reason: Option<TrapReason>,
	/// Nested contract execution context
    nest: Vec<NestedRuntime>,
}
```
