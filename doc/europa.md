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
For using all features when running contracts in Europa, we advice developers use [Patract Labs's `cargo-contract`](https://github.com/paritytech/cargo-contract)
to compile ink! contract, until [this pr#131 Enable debug info to the source warehouse with flag in command build](https://github.com/paritytech/cargo-contract/pull/131) could be merged by paritytech.

In Patract Labs's `cargo-contract`, we will contain the "name section" while compile contracts. Before this PR is merged, 
currently, only the `cargo-contract` version provided by us (Patract Labs) can be used:

```bash
cargo install --git https://github.com/patractlabs/cargo-contract --branch v0.10.0 --force
```

If you do not want this version of `cargo-contract` to override the version released by paritytech, then it is recommended 
to compile locally and use the compiled `cargo-contract` directly:

```bash
git clone https://github.com/patractlabs/cargo-contract --branch v0.10.0
cd cargo-contract
cargo build --release
```

> Note: Executing the `cargo-contract build` command requires the `default toolchain` of the rust toolchain to be `nightly`, 
> otherwise you can only use `cargo +nightly contract build`, but using `cargo` to call `cargo-contract` needs to be 
> executed `cargo install` installs or overwrites the compiled product in the `~/.cargo/bin` directory, and cannot co-exist 
> with paritytech's `cargo-contract`

Execute:

```bash
cargo-contract build --help
# or
cargo +nightly contract build --help
```

If you can see:

```bash
FLAGS:
    -d, --debug      
            Emits debug info into wasm file
```

It means that you are using the `cargo-contract` provided by Patract Labs. If you want to see the backtrace of the WASM 
contract execution crash while using Europa, you need to add the `--debug` command when compiling the contract.

Using the `--debug` command will generate file in the `target/ink` directory of the originally compiled contract, 
ending with `*.wasm`. This `*.wasm` file is the WASM contract file containing the "name section" part.

**If you need to use Europa for testing, the contract deployed to Europa needs to use this `*.wasm` file instead of the originally generated `*opt.wasm` file.**

> In following doc, about the log part, if the contract do not have "name section" (contracts are not compiled by `--debug`
> or not submit `*.wasm` file), the output may contain a batch of `<unknown>`. If you meet this, please use the contract
> which has "name section".
> ```bash
> wasm_error: Error::WasmiExecution(Trap(Trap { kind: Unreachable }))
>    wasm backtrace:
>    |  <unknown>[...]
>    |  <unknown>[...]
>    ╰─><unknown>[...]
> ```
 
### Design and examples
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
Currently, the recorded information in this struct **is printed every time while the contract be executed** (including from 
rpc call or extrinsic). In the future, this data could be stored in local and access by rpc call for 3rd-parity client, 
which can be used for visualized presentation to show the detailed information in execution contract.

In the model of `pallet-contracts`, a contract calling another contract is in the "contract stack" model, so `NestedRuntime` 
will track the execution process of the entire contract stack, and use the property of `nest` to store a list of `NestedRuntime`
to represent other contracts the the contract called.

In the process of executing a contract by `pallet-contracts`, Europa records the relevant information in the execution 
process in the structure of `NestedRuntime` in the form of a bypass, and will print the `NestedRuntime` to the log 
(show the case later) in a certain format after the contract call ends. Contract developers can analyze the information
printed by `NestedRuntime` to obtain various detailed information during the execution of the contract, which can be used 
in various situations:

1. help to locate where the error occurs, including the following situations:
    1. `pallet-contracts` layer
    2. `ink!` layer
    3. The specific position in the contract layer
    4. Locate which level of the contract is when a contract calling another contract
2. Analyze the information during the execution of the contract at this timing:
    1. Analyze the consumption of gas execution
    2. Analyze the call of `get_storage` and `set_storage`, help reconstruct the contract code and analyze the demand of `rent`
    3. According to `selector`, `args` and `value`, analyze and locate whether the transaction parameters of the third-party SDK are legal.
    4. Analyze the execution path of the contract and adjust the contract based on the `nest` information and combined with the `seal_call` information.
    5. etc.

The process of recording `pallet-contracts` executing contract to `NestedRuntime` is relatively fine-grained.
The process of logging the information of the execution contract of `pallet-contracts` to `NestedRuntime` is relatively fine-grained. Take `seal_call` in `define_env!` as an example:

```rust
pub struct SealCall {
    callee: Option<HexVec>,
    gas: u64,
    value: Option<u128>,
    input: Option<HexVec>,
    output: Option<HexVec>,
}
```

The attributes are basically `Option<>`. For example, before calling the contract, the `input` will be set to `Some`, 
and the return value will be set after the calling contract is normal. If there is an error in the calling contract, 
then `output` will remain `None`. Therefore, if `input` is `Some` and `output` is `None`, it means that there is a 
problem with the called contract during the process of calling the contract.

The example log print like this, this log is printed when the `ink/example/flipper` contract's `get` message is called 
by rpc request `contracts_call`:

```bash
1: NestedRuntime {
    ext_result: [success] ExecReturnValue { flags: 0, data: 01 },
    caller: 0000000000000000000000000000000000000000000000000000000000000000 (5C4hrfjw...),
    self_account: 3790ddf4d8c63d559b3b46b96ca9b7b5f07b772c9ad4587eca6c0738e5d48422 (5DKZXRQN...),
    selector: 0x1e5ca456,
    args: None,
    value: 0,
    gas_limit: 4999999999999,
    gas_left: 4998334662707,
    env_trace: [
        seal_value_transferred(Some(0x00000000000000000000000000000000)),
        seal_input(Some(0x1e5ca456)),
        seal_get_storage((Some(0x0000000000000000000000000000000000000000000000000000000000000000), Some(0x01))),
        seal_return((0, Some(0x01))),
    ],
    trap_reason: TrapReason::Return(ReturnData { flags: 0, data: 01 }),
    nest: [],
}
```
Let's explain the information printed above:

1. `ext_result`: indicates that this contract call is displayed as successful or failed:

    1. `[success]`: indicates the successful execution of this contract (Note: the successful execution of the contract
       does not mean the successful execution of the business logic of the contract itself. There may be an error return 
       in `ink!` or the business logic of the contract itself, as in case 3 in the following text.) And the `ExecResultValue {flag:0, data: 0x...}` 
       followed by `[success]` indicates the return value after this contract is executed.
    2. `[failed]`: indicates that the execution of this contract failed, and the `ExecError {.. }` followed by `[failed]` 
       indicates the cause of this error. The reason for this is the value recorded in `event` on the chain, which is the 
       value defined in `decl_error!` of `pallet-contracts`.

2. `1: NestedRuntime` & `nest`: The contract information that represents the current print information is located in the 
   first layer of the contract call stack. If the current contract calls another contract, it will appear in the array 
   of the `nest` field. `2: NestedRuntime` and `1: NestedRuntime` has the same structure. Among them, `2` indicates that 
   the called contract is in the second layer of the contract call stack. If several contracts are called across contracts 
   in the current contract, there will be several `NestedRuntime` in the array of `nest`. If there are other contract calls 
   in the second-level contract, the same goes for.

   For example, if there are contracts A, B, C, if it is the following situation:

    1. After A calls B, it returns to A to continue execution, and then calls contract C

       ![call_other_1](/Users/jenner/codes/substrate-contracts-book/src/zh_CN/europa/img/call_other_1.png)

       Then it will produce a log print similar to the following:

       ```text
       1: NestedRuntime {
        self_account: A,
        nest:[
            2: NestedRuntime {
                self_account: B,
                nest:[],
            },
            2: NestedRuntime {
                self_account: C,
                nest:[],
            }
        ]
       }
       ```

    2. After A calls B, B calls contract C again, and finally returns to A

       ![call_other_2](/Users/jenner/codes/substrate-contracts-book/src/zh_CN/europa/img/call_other_2.png)

       Then it will produce a log print similar to the following:

       ```text
       1: NestedRuntime {
        self_account: A,
        nest:[
            2: NestedRuntime {
                self_account: B,
                nest:[
                    3: NestedRuntime {
                       self_account: C,
                       nest:[],
                   }
                ],
            }  
        ]
       }
       ```

3. `caller`: who is the caller of the current contract. If the contract calls the contract, the value of the called 
   contract is the address of the upper-level contract. (The addr is `0x000000...` for this example is called by rpc.)
   
4. `self_account`: represents the address of the current contract itself.

5. `selector` & `args`&`value`: Represents the `selector` and parameters passed in when calling the current contract. 
   **These information can quickly locate whether the calling contract method is correct**.

6. `gas_limit` & `gas_left`: Represents the `gas_limit` passed in when the contract is currently called and the remaining 
   gas after **executing this layer**. Note here that `gas_left` refers to the remaining gas after the execution of this 
   layer of contract, so In the contract call contract, the gas consumed by each layer of contract can be determined
   through `gas_left`, not only get the consumption after the execution of the entire contract.

7. `env_trace`: Indicates that during the execution of the current layer of the contract, each time host_function is called 
   in the contract WASM execution, a record will be added to the list here. Because all host_functions and the definitions
   in [`define_env!` in the `pallet-contracts` module](https://github.com/paritytech/substrate/blob/master/frame/contracts/src/wasm/runtime.rs#L610 ) are related, 
   so tracing `env_trace` can trace the process of interacting with `pallet-contracts` during the execution of the current WASM contract.

   For example, if following thing appears in `env_trace`:

    - `seal_call`: It means that there is a contract call contract situation in the current contract. According to the
      order in which `seal_call` appears in `env_trace`, it can correspond to `nest` to calculate the state before and 
      after the contract calls the contract.
    - `seal_get_storage`&`seal_set_storage`: It means that data read and write occurred in the contract. Through these 
      two interfaces, it is possible to intercept and count the data read and write during the execution of the current 
      contract, and the data size calculated by **`seal_set_storage` can also be used to infer the storage size required by `rent`**.
    - `seal_deposit_event`: indicates that the event is printed in the contract. Here you can intercept the content of 
      each event separately, instead of getting a unified event at the end. And the following text will use an example 
      to surface that Europa can quickly locate the bug in the `host_function`.

   On the other hand, the statistics of `env_trace` are relatively **fine-grained**. For example, if there are multiple 
   possible errors in `host_function`, when an error occurs, all the information before the error will be retained, 
   so it can be located to the place where the problem occurred during the execution of `host_function`.

   And if there is an error in `host_function` that causes the contract to end execution, `env_trace` records the last 
   error `host_function` call, so you can directly locate which `host_function` caused the contract execution exception.

8. `trap_reason`: According to the definition of `TrapReason` in `pallet-contracts`, `trap_reason` can be divided into 2 categories:

    1. `Return` & `Termination` & `Restoration`: indicates that the contract exit is the design of `pallet-contracts`, 
       not an internal error. This type of trap indicates that the contract is executed normally and is not an error.
    2. `SupervisorError`: Indicates that an error occurred during the execution of the contract calling host_function.

   Therefore, the current Europa log printing design is designed to record whenever `trap_reason` appears. On the other hand, 
   `trap_reason` may not always appear during the execution of the contract. Combining the design of `pallet-contracts` and `ink!`, 
   there is a case where the successful execution of the contract or the execution failure in the `ink!` layer does not 
   generate `trap_reason`. Therefore, in addition to recording `trap_reason`, Europa also **records the results returned
   by the WASM executor after execution, which is recorded with `sandbox_result_ok`.**

9. `sandbox_result_ok`: The value of `sandbox_result_ok` represents the result of the contract after the WASM executor is 
   executed. This value could have been recorded as `sandbox_result`, including correct and incorrect conditions. However, 
   due to the limitations of Rust and combined with the business logic of `pallet-contracts`, only the result of `sandbox_result` 
   is kept as `Ok` here. **For log printing, Europa is designed to print `sandbox_result_ok` only when trap_reason is the
   first case, as information to assist in judging contract execution.**

   `sandbox_result_ok` is the WASM executor [result after calling `invoke`](https://github.com/paritytech/substrate/blob/712085115cdef4a79a66747338c920d6ba4e479f/frame/contracts/src/wasm/mod.rs#L155-L156) 
   After the processing of `to_execution_result`, if there is no `trap_reason`, the result of `Ok(..)` is [discarded](https://github.com/paritytech/substrate/blob/712085115cdef4a79a66747338c920d6ba4e479f/frame/contracts/src/wasm/runtime.rs#L366-L368). 
   But in fact there are two situations here:

    1. An error occurred in `ink!`: According to the implementation of `ink!`, before calling the functions wrapped by 
       the contract `#[ink(message)]` and `#[ink(constructor)]`, the input The process of decoding and matching `selector`. 
       If an error occurs during this process, the contract will return [error code `DispatchError`](https://github.com/paritytech/ink/blob/abd5cf14c0883cb2d5acf81f2277aeec330aa843/crates/lang/src/error.rs#L22). 
       But for the WASM executor, the WASM code is executed normally, so the result will be returned, including this error code. 
       **This contract execution process is an error situation.**
    2. The return value of `#[ink(message)]` is defined as `()`: According to the implementation of `ink!`, if the return 
       value type is `()`, `seal_reason` will not be called, so it will not Contains `trap_reason`. **This contract execution 
       process is an correct situation.**

   Since `ink!` is only a contract implementation that runs on `pallet-contracts`, other implementations may have different rules, 
   so currently `sandbox_result_ok` is only used to assist in determining the execution of the `ink!` contract, the value
   is [` ReturnValue`](https://github.com/paritytech/substrate/blob/712085115cdef4a79a66747338c920d6ba4e479f/primitives/wasm-interface/src/lib.rs#L462-L467). 
   Among them, if the `<num>` part of `ReturnValue::Value(<num>)` of the log is not 0, it means that there may be an error 
   in the execution of `ink!`. You can use `ink!` for [`DispatchError` The error code](https://github.com/paritytech/ink/blob/abd5cf14c0883cb2d5acf81f2277aeec330aa843/crates/lang/src/error.rs#L66-L80) 
   determines the error.

10. `wasm_error`: indicates the backtrace when WASM executes an error. This part will be printed only when `ext_result` is `failed`.

    The rpc call in this example is called normally, thus there is no `wasm_error` field. We will show more example later.

#### 2. `wasmi` panic backtrace

We forked wasmi and integrated it into `ep-sandbox`. Forked `pallet-contracts` can obtain the error information of forked `wasmi` through `ep-sandbox`, including the backtrace information of `wasmi`.

If you need to make `wasmi` can retain the backtrace information during execution, you need to have the following functions:

1. The "name section" section is required in the WASM source file (see [the specification of name section)](https://webassembly.github.io/spec/core/appendix/custom.html#name-section))
2. Keep the "name section" information in the process of checking the contract by `pallet-contracts` and still have a corresponding relationship with the wasm source file after the process.
3. During the execution of `wasmi`, the execution stack needs to be preserved with the key information of the functions. At the same time, the "name section" needs to be parsed and correspond to the function information reserved by the `wasmi` execution stack.

The changes to 2 involve `cargo-build` and `parity-wasm`, while the changes to 1 and 3 are mainly in the forked `wasmi`, and a small part involves `pwasm-utils`.

And in all, we create following pr to the origin repo:
- `cargo-contract`
    - PR: [paritytech/cargo-contract#131](https://github.com/paritytech/cargo-contract/pull/131)
    - Source: [patractlabs/cargo-contract](https://github.com/patractlabs/cargo-contract)
- `parity-wasm`
    - PR: [paritytech/parity-wasm#300](https://github.com/paritytech/parity-wasm/pull/300)
- `wasm-utils`
    - PR: [paritytech/wasm-utils#146](https://github.com/paritytech/wasm-utils/pull/146)
    - Source: [patractlabs/wasm-utils#146](https://github.com/patractlabs/wasm-utils)
- `wasmi`
    - PR: No pr for this repo yet
    - Source: [patractlabs/wasmi](https://github.com/patractlabs/wasmi)

##### 2.1 `wasmi` panic backtrace example

For example, we modify the example contract `ink/example/erc20` in the [ink!](https://github.com/paritytech/ink) project as follows:

```rust
#[ink(message)]
pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
    let from = self.env().caller();
    self.transfer_from_to(from, to, value)?;
    panic!("123");
    Ok(())
}
```

WASM, it corresponds to the code after the macro expansion of the original file, so if you want to compare the errors of the call stack, you need to expand the macro of the original contract:

```bash
cargo install expand
cd ink/example/erc20
cargo expand > tmp.rs
```

After reading the `tmp.rs` file, we can know that WASM needs to go through when it executes the `transfer` function:

```bash
fn call() -> u32 
-> <Erc20 as ::ink_lang::DispatchUsingMode>::dispatch_using_mode(...)
-> <<Erc20 as ::ink_lang::ConstructorDispatcher>::Type as ::ink_lang::Execute>::execute(..)  # compile selector at here
-> ::ink_lang::execute_message_mut
-> move |state: &mut Erc20| { ... } # a closure
-> <__ink_Msg<[(); 2644567034usize]> as ::ink_lang::MessageMut>::CALLABLE
-> transfer
```

Therefore, if the `panic` in `transfer` is encountered during the contract call, the backtrace of WASM should be similar to this.

After putting code and deploying the contract, if developer calls the `transfer` message, the Europa will print:
```bash
1: NestedRuntime {
	ext_result: [failed] ExecError { error: DispatchError::Module {index:5, error:17, message: Some("ContractTrapped"), orign: ErrorOrigin::Caller }}
    caller: d43593c715fdd31c61141abd04a99fd6822...(5GrwvaEF...),
    self_account: b6484f58b7b939e93fff7dc10a654af7e.... (5GBi41bY...),
    selector: 0xfae3a09d,
    args: 0x1cbd2d43530a44705ad088af313e18f80b5....,
    value: 0,
    gas_limit: 409568000000,
    gas_left: 369902872067,
    env_trace: [
        seal_value_transferred(Some(0x00000000000000000000000000000000)),
        seal_input(Some(0xfae3a09d1cbd.....)),
        seal_get_storage((Some(0x0100000000000....), Some(0x010000000100000001000000))),
        # ...
        seal_caller(Some(0xd43593c715fdd31c61141abd...)),
        seal_hash_blake256((Some(0x696e6b20686173....), Some(0x0873b31b7a3cf....))),
      	# ...  
        seal_deposit_event((Some([0x45726332303a....00000000000]), Some(0x000..))),
    ],
	trap_reason: TrapReason::SupervisorError(DispatchError::Module { index: 5, error: 17, message: Some("ContractTrapped") }),
    wasm_error: Error::WasmiExecution(Trap(Trap { kind: Unreachable }))
        wasm backtrace: 
        |  core::panicking::panic[28]
        |  erc20::erc20::_::<impl erc20::erc20::Erc20>::transfer[1697]
        |  <erc20::erc20::_::__ink_Msg<[(); 2644567034]> as ink_lang::traits::MessageMut>::CALLABLE::{{closure}}[611]
        |  core::ops::function::FnOnce::call_once[610]
        |  <erc20::erc20::_::_::__ink_MessageDispatchEnum as ink_lang::dispatcher::Execute>::execute::{{closure}}[1675]
        |  ink_lang::dispatcher::execute_message_mut[1674]
        |  <erc20::erc20::_::_::__ink_MessageDispatchEnum as ink_lang::dispatcher::Execute>::execute[1692]
        |  erc20::erc20::_::<impl ink_lang::contract::DispatchUsingMode for erc20::erc20::Erc20>::dispatch_using_mode[1690]
        |  call[1691]
        ╰─><unknown>[2387]
    ,
    nest: [],
}
```

In the above example, because the execution of `transfer` will trigger `panic`, you can see that the cause of the error 
here is `WasmiExecution(Trap(Trap {kind: Unreachable }))`, indicating that this time the failure is due to execution 
The situation of `Unreacble` in the contract process is caused, and the backtrace information below also **very accurately describes** 
the function execution call stack when an error is encountered after the expansion of the contract macro discussed above. 
The following calling process can be clearly found from the backtrace.

```text
call -> dispatch_using_mode -> ... -> transfer -> panic 
```
This process is consistent with the original information of the contract.

#### 3. ChainExtensions
##### 3.1 ink logger
More information refers to [ink-log](https://github.com/patractlabs/ink-log).

##### 3.2 ZKP feature
More information refers to [megaclite](https://github.com/patractlabs/megaclite), and the example contracts in [metis/groth16](https://github.com/patractlabs/metis/tree/master/groth16).



#### Other examples:
##### Example 1：`ContractTrap` caused by locating duplicate topics

Some time ago, we (Patract Labs) reported a bug to `ink!`, see issue:["When set '0' value in contracts event, may cause `Error::ContractTrapped` and panic in contract #589" ](https://github.com/paritytech/ink/issues/589). It is very difficult to locate this error before Europa has developed the relevant function. Thank you @athei [located the error](https://github.com/paritytech/ink/issues/589#issuecomment-731571918). Here we reproduce this error and use Europa's log to quickly analyze and locate the place where the bug appears:

1. checkout `ink!` to commit `8e8fe09565ca6d2fad7701d68ff13f12deda7eed`

   ```bash
   cd ink
   git checkout 8e8fe09565ca6d2fad7701d68ff13f12deda7eed -b tmp
   ```

2. Go in `ink/examples/erc20/lib.rs:L90` to change `value` to `0_u128` in `Transfer`

   ```rust
   #[ink(constructor)]
   pub fn new(initial_supply: Balance) -> Self {
   	//...
       Self::env().emit_event(Transfer {
           from: None,
           to: Some(caller),
           // change this from `initial_supply` to `0_u128`
           value: 0_u128.into() // initial_supply,
       });
       instance
   }
   ```

3. Execute `cargo +nightly contract build --debug` to compile the contract

4. Use [RedSpot](https://redspot.patract.io/en/tutorial/) or [`Polkadot/Substrate Portal`](https://polkadot.js.org/apps) to deploy this contract ( Note that you must use the erc20.src.wasm file)

You should encounter `DuplicateTopics` in the deployment phase (before this [bug](https://github.com/paritytech/substrate/pull/7762) is corrected, the reported error is `ContractTrap`), and in the Europa log Will show:

```bash
1: NestedRuntime {
    #...
    env_trace: [
        seal_input(Some(0xd183512b0)),
		#...    
		seal_deposit_event((Some([0x45726332303a3a5472616e736....]), None)),
    ],
    trap_reason: TrapReason::SupervisorError(DispatchError::Module { index: 5, error: 23, message: Some("DuplicateTopics") }),
    wasm_error: Error::WasmiExecution(Trap(Trap { kind: Host(DummyHostError) }))
    	wasm backtrace: 
    	|  ink_env::engine::on_chain::ext::deposit_event[1623]
    	|  ink_env::engine::on_chain::impls::<impl ink_env::backend::TypedEnvBackend for ink_env::engine::on_chain::EnvInstance>::emit_event[1564]
    	|  ink_env::api::emit_event::{{closure}}[1563]
    	|  <ink_env::engine::on_chain::EnvInstance as ink_env::engine::OnInstance>::on_instance[1562]
    	|  ink_env::api::emit_event[1561]
    	|  erc20::erc20::_::<impl ink_lang::events::EmitEvent<erc20::erc20::Erc20> for ink_lang::env_access::EnvAccess<<erc20::erc20::Erc20 as ink_lang::env_access::ContractEnv>::Env>>::emit_event[1685]
# ...
# ...
    	|  deploy[1691]
    	╰─><unknown>[2385]
    ,
    nest: [],
}
```

You can see from the above log:

1. The last record of `env_trace` is `seal_deposit_event` instead of `seal_return` (when the contract is executed correctly, the last record must be `seal_return`)
2. The second parameter of `seal_deposit_event` is `None` instead of an existing value, which indicates that the host_function of `seal_deposit_event` has not been executed, but an error occurred during the execution (see the forked dependency of Europa) See the [corresponding implementation] (https://github.com/patractlabs/substrate/blob/3624deb47cabe6f6cd44ec2c49c6ae5a29fd2198/frame/contracts/src/wasm/runtime.rs#L1399) for the source code of the version of `Pallet Contracts`.
3. Combined with the error stack of wasm backtrace, we can intuitively see that the top call stack of backtrace is `deposit_event`.

Therefore, combining the above information, we can directly infer that the host_function of `seal_deposit_event` has an exception during the execution. (Before the submission of `Pallet Contracts`[pull#7762](https://github.com/paritytech/substrate/pull/7762), we recorded the error message in host_function. After the merge, we used `trap_reason` unified error message.)

##### Example 2: When error is caused by the chain using `type Balance=u64` instead of `type Balance=u128`

If the chain uses the definition of `Balance=u64`, and the definition of `Balance` in the chain is unknown to `ink!` (the default definition of Balance is `u128`). Therefore, when using `u128` to define `Balance`'s `ink!` as a dependency compiled contract, when running on a chain where `Balance` is defined as `u64`, it will cause the `Pallet Contracts` module to transfer values to the contract , The contract internally regards the `value` of `u64` as a decoding error of `u128`.

Take the example contract of erc20 as an example, after expanding the macro of the contract, you can see:

In the call of `call`, since `deny_payment` is checked before calling `dispatch_using_mode`, and if an Error is returned when checking `deny_payment`, it will be directly `panic`.

Therefore, in this case, the contract for deploying (`Instantiate`) ERC20 will execute normally, and any method of ERC20 such as `transfer` will be called with `ContractTrap`.

The `call` stage, such as calling `transfer`:

   Calling `transfer` to the above successfully instantiated function, `ContractTrap` will appear, Europa's log shows as follows:

   ```bash
   1: NestedRuntime {
   	ext_result: [failed] ExecError { error: DispatchError::Module {index:5, error:17, message: Some("ContractTrapped"), orign: ErrorOrigin::Caller }}
   # ...
       env_trace: [
           seal_value_transferred(Some(0x0000000000000000)),
       ],
       wasm_error: Error::WasmiExecution(Trap(Trap { kind: Unreachable }))
       	wasm backtrace: 
       	|  core::panicking::panic_fmt.60[1743]
       	|  core::result::unwrap_failed[914]
       	|  core::result::Result<T,E>::expect[915]
       	|  ink_lang::dispatcher::deny_payment[1664]
       	|  call[1691]
       	╰─><unknown>[2387]
       ,
       nest: [],
   }
   ```

   First notice that the last record of `env_trace` is still not `seal_return`, and the error cause of `wasm_error` is `WasmiExecution::Unreachable`. Therefore, it can be determined that `panic` or `expect` was encountered during the execution of the contract.

   From the wasm backtrace, it is very obvious that the execution process is

```bash
   call -> deny_payment -> expect
```

According to the code expanded macro (`cd ink/examples/erc20; cargo expand> tmp.rs`), we can see:

   ```bash
   #[no_mangle]
fn call() -> u32 {
       if true {
        ::ink_lang::deny_payment::<<Erc20 as ::ink_lang::ContractEnv>::Env>()
       		.expect("caller transferred value even though all ink! message deny payments")
       }
       ::ink_lang::DispatchRetCode::from(
           <Erc20 as ::ink_lang::DispatchUsingMode>::dispatch_using_mode(
               ::ink_lang::DispatchMode::Call,
           ),
       )
       .to_u32()
   }
   ```

Therefore, it can be judged that an error was returned in `deny_payment` during the execution of the contract in the process of `transfer`, and the direct processing of the error as `expect` resulted in the execution result of `wasmi` being `Unreachable` Tracking the code of `deny_payment` can find that the function returns `expect` caused by `Error`

> Note，The relevant code is as follows:
>
> In `ink_lang` https://github.com/paritytech/ink/blob/master/crates/lang/src/dispatcher.rs#L140-L150
>
> ```rust
> pub fn deny_payment<E>() -> Result<()>
> where
>  E: Environment,
> {
>  let transferred = ink_env::transferred_balance::<E>()
>      .expect("encountered error while querying transferred balance");
>  if transferred != <E as Environment>::Balance::from(0u32) {
>      return Err(DispatchError::PaidUnpayableMessage)
>  }
>  Ok(())
> }
> ```
>
> **There will be a difference between the `off_chain` part and the `on_chain` part in the ink**, `off_chain` will think that an error is returned at the stage of `ink_env::transferred_balance::<E>()`, so it is executing` After transferred_balance`, you will encounter `expect` which leads to `panic`, and part of `on_chain` is taken from the memory of wasm, it will normally get the characters corresponding to u128 length and decode to get `transferred`, which is just decoded The result will not meet expectations, causing `transferred!=0` to make `deny_payment` return an Error, and the part where `deny_payment` is called in the macro expansion of the contract triggers `expect`
>
> ```rust
> if true {
>  ::ink_lang::deny_payment::<<Erc20 as ::ink_lang::ContractEnv>::Env>()
>  	.expect("caller transferred value even though all ink! message deny payments")
> }
> ```
>
> Therefore, for wasm backtrace, `expect` appears when `deny_payment` is called in `call`, not when `transferred_balance` is called in `deny_payment`.
>
> **This example side shows that `ink!` currently does not completely correspond to the processing of `off_chain` and `on_chain`, and may cause difficult-to-check errors for contract users in some cases**
